#!/usr/bin/env python3
"""Exercise an Electrum broadcast and retry using an isolated Bitcoin regtest.

Requires bitcoind, bitcoin-cli, Cargo, and electrs/target/release/electrs.
All chain and wallet data are created in a temporary directory and removed.
"""

import json
import os
import random
import socket
import subprocess
import tempfile
import time
from pathlib import Path

os.chdir(Path(__file__).resolve().parent.parent)


def free_port(low):
    for _ in range(1000):
        port = random.randrange(low, low + 5000)
        with socket.socket() as sock:
            try:
                sock.bind(("127.0.0.1", port))
                return port
            except OSError:
                continue
    raise RuntimeError("No available port")


def stop(process):
    if process is None or process.poll() is not None:
        return
    process.terminate()
    try:
        process.wait(timeout=15)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()


rpc, p2p, electrum, monitor = [free_port(x) for x in (20000, 30000, 40000, 50000)]
with tempfile.TemporaryDirectory(prefix="satellion-regtest-") as tmp:
    root = Path(tmp)
    bitcoin_dir = root / "bitcoin"
    bitcoin_dir.mkdir()
    with (root / "node.log").open("w+") as node_log, (root / "electrs.log").open("w+") as electrs_log:
        node = subprocess.Popen(
            [
                "bitcoind", "-regtest", f"-datadir={bitcoin_dir}", f"-rpcport={rpc}",
                f"-port={p2p}", f"-bind=127.0.0.1:{p2p}", "-server=1", "-txindex=1",
                "-fallbackfee=0.0001", "-dnsseed=0", "-connect=0", "-discover=0",
                "-listenonion=0",
            ],
            stdout=node_log, stderr=subprocess.STDOUT,
        )
        index = None

        def cli(*args):
            result = subprocess.run(
                ["bitcoin-cli", "-regtest", f"-datadir={bitcoin_dir}", f"-rpcport={rpc}", *args],
                capture_output=True, text=True, timeout=30,
            )
            if result.returncode:
                raise RuntimeError(result.stderr.strip())
            return result.stdout.strip()

        try:
            for _ in range(100):
                try:
                    cli("getblockchaininfo")
                    break
                except RuntimeError:
                    if node.poll() is not None:
                        raise RuntimeError("bitcoind exited")
                    time.sleep(0.1)
            else:
                raise RuntimeError("bitcoind startup timeout")

            cli("createwallet", "satellion-test")
            cli("generatetoaddress", "101", cli("getnewaddress"))
            utxo = json.loads(cli("listunspent", "1"))[0]
            destination = cli("getnewaddress")
            raw = cli(
                "createrawtransaction",
                json.dumps([{"txid": utxo["txid"], "vout": utxo["vout"]}]),
                json.dumps({destination: 49.999}),
            )
            signed = json.loads(cli("signrawtransactionwithwallet", raw))
            if not signed["complete"]:
                raise RuntimeError("Signing incomplete")

            index = subprocess.Popen(
                [
                    "electrs/target/release/electrs", "--skip-default-conf-files",
                    "--network", "regtest", "--daemon-dir", str(bitcoin_dir),
                    "--db-dir", str(root / "index"), "--daemon-rpc-addr", f"127.0.0.1:{rpc}",
                    "--daemon-p2p-addr", f"127.0.0.1:{p2p}", "--electrum-rpc-addr",
                    f"127.0.0.1:{electrum}", "--monitoring-addr", f"127.0.0.1:{monitor}",
                    "--wait-duration-secs", "1",
                ],
                stdout=electrs_log, stderr=subprocess.STDOUT,
            )
            for _ in range(300):
                if index.poll() is not None:
                    raise RuntimeError("electrs exited")
                try:
                    with socket.create_connection(("127.0.0.1", electrum), timeout=0.2):
                        break
                except OSError:
                    time.sleep(0.1)
            else:
                raise RuntimeError("electrs startup timeout")

            env = os.environ.copy()
            env["SATELLION_TEST_ELECTRUM"] = f"127.0.0.1:{electrum}"
            env["SATELLION_TEST_TX_HEX"] = signed["hex"]
            result = subprocess.run(
                [
                    "cargo", "test", "--manifest-path", "src-tauri/Cargo.toml", "--lib",
                    "regtest_broadcast_and_retry_same_transaction", "--", "--ignored",
                ],
                env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=120,
            )
            print(result.stdout[-4000:])
            if result.returncode:
                raise RuntimeError("Regtest failed")
            mempool = json.loads(cli("getrawmempool"))
            if len(mempool) != 1:
                raise RuntimeError(f"Expected one mempool transaction, got {len(mempool)}")
            print("Mempool contains exactly one transaction")
        except Exception:
            for name, logfile in (("electrs", electrs_log), ("node", node_log)):
                logfile.flush()
                logfile.seek(0)
                print(f"{name} log tail:", logfile.read()[-3000:])
            raise
        finally:
            stop(index)
            stop(node)
