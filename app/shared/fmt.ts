export function fmt_timestamp(ts: number): string {
    let dt = new Date(ts * 1e3)

    let date = `${dt.getFullYear()}/${dt.getMonth()}/${dt.getDate()}`
    let time = `${dt.getHours()}:${dt.getMinutes()}:${dt.getSeconds()}`

    return date + ' - ' + time
}

export function fmt_timeago(ts: number): string {
    if (ts == 0) return '---'
    let seconds = ~~(new Date().getTime() / 1e3) - ts
    if (seconds < 1) return 'now'

    let out = ''

    if (seconds > 2592000) {
        let months = ~~(seconds / 2592000)
        seconds = seconds - months * 2592000
        out += months + 'M '
    }

    if (seconds > 86400) {
        let days = ~~(seconds / 86400)
        seconds = seconds - days * 86400
        out += days + 'd '
    }

    if (seconds > 3600) {
        let hours = ~~(seconds / 3600)
        seconds = seconds - hours * 3600
        out += hours + 'h '
    }

    if (seconds > 60) {
        let minutes = ~~(seconds / 60)
        seconds = seconds - minutes * 60
        out += minutes + 'm '
    }

    if (seconds > 0) {
        out += seconds + 's '
    }

    return out + 'ago'
}
