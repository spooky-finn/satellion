# Satellion - Bitcoin & Ethereum non-custodial wallet

⚠️🚧 This project is still under construction and expected to change significantly. Use at your own risk.

## MacOS buypass Apple notarization check

```bash
xattr -dr com.apple.quarantine /Applications/Satellion.app
```

## Frontend development

The desktop frontend uses Svelte 5, Skeleton UI 5, and Tailwind CSS 4 with Vite.
State lives in reactive classes in `*.svelte.ts` modules: mutable fields use
`$state`, derived values use getters, and asynchronous results that are replaced
as a whole use `$state.raw`. There is no external state management library.

`src/routes/` contains screens and chain-specific view models. Shared session
state lives in `src/view_model/`, reusable controls in `src/components/`, and
request, routing, and theme helpers in `src/lib/`.

Route components compose domain sections and dialogs. Wallet creation and unlock
steps live in `src/routes/wallet/`, settings controls in `src/routes/settings/`,
and chain operations beside their wallet screens. Pass view models and callbacks
to these components; keep access to `root_store` at the route boundary.

Route-local reactive classes own workflow state and expose cleanup explicitly.
Creation clears recovery secrets on unmount; unlock ignores completion after
the route is disposed. Nested dialogs stay inside their parent's focus scope.

Separate imports, state, methods, and request phases with a blank line. Keep
related state fields together, and use short comments to explain request guards,
cleanup, or transaction review requirements.

```bash
pnpm install
pnpm tauri dev      # Desktop app with wallet commands
pnpm dev            # Frontend preview at localhost:1420
pnpm check          # Svelte and TypeScript diagnostics
pnpm build          # Validate and generate the desktop frontend in dist/
pnpm fmt            # Format frontend files with Biome
pnpm fmt:check      # Check formatting without changing files
pnpm fix            # Apply Biome lint, import, and formatting fixes
```

Biome formats Svelte, TypeScript, JavaScript, CSS, HTML, and JSON. Svelte
formatting uses Biome's experimental full HTML support, and Tailwind CSS 4
directives are enabled. Generated bindings, backend files, and the `electrs`
submodule are outside its scope. The pre-commit hook runs `pnpm fmt`.

The custom Skeleton theme in `src/theme.css` preserves the original warm light
and dark palettes.

The browser preview displays a desktop-app notice because wallet operations
require Tauri. Run the desktop app to validate native IPC, Touch ID, and chain
connectivity.

`src/bindings/` contains generated Tauri command types and should not be edited
manually. Vite keeps the desktop app's fixed port and HMR configuration.
