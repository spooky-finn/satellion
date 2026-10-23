const ErrMsgTime = 10000
const OkMsgTime = 3000

type Notification = {
  id: string
  msg: string
  level: 'err' | 'info'
}

/** Shared toast messages that expire independently after their timeout. */
class NotifierStore {
  notifications = $state<Notification[]>([])

  private notify(level: Notification['level'], msg: string, timeout: number) {
    const id = crypto.randomUUID()

    this.notifications.push({ msg, level, id })
    setTimeout(() => {
      this.notifications = this.notifications.filter(each => each.id !== id)
    }, timeout)
  }

  list() {
    return this.notifications.toReversed()
  }

  ok(msg: string, timeout?: number) {
    this.notify('info', msg, timeout || OkMsgTime)
  }

  err(msg: string, timeout?: number) {
    this.notify('err', String(msg), timeout || ErrMsgTime)
  }
}

export const notifier = new NotifierStore()
