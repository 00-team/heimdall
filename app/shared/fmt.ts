export function fmt_timestamp(ts: number): string {
    let dt = new Date(ts * 1e3)

    let month = (dt.getMonth() + 1).toString().padStart(2, '0')
    let date = `${dt.getFullYear()}/${month}/${dt.getDate()}`
    let time = `${dt.getHours()}:${dt.getMinutes()}:${dt.getSeconds()}`

    return date + ' - ' + time
}

export function fmt_duration(seconds: number): string {
    if (seconds < 1) return 'instant'

    let out = ''

    if (seconds >= 2592000) {
        let months = ~~(seconds / 2592000)
        seconds = seconds - months * 2592000
        out += months + 'M '
    }

    if (seconds >= 86400) {
        let days = ~~(seconds / 86400)
        seconds = seconds - days * 86400
        out += days + 'd '
    }

    if (seconds >= 3600) {
        let hours = ~~(seconds / 3600)
        seconds = seconds - hours * 3600
        out += hours + 'h '
    }

    if (seconds >= 60) {
        let minutes = ~~(seconds / 60)
        seconds = seconds - minutes * 60
        out += minutes + 'm '
    }

    if (seconds > 0) {
        out += seconds + 's '
    }

    return out
}

export function fmt_timeago_ts(ts: number): string {
    return fmt_timeago(~~(new Date().getTime() / 1e3) - ts)
}
export function fmt_timeago(seconds: number): string {
    if (seconds < 1) return 'now'

    let out = ''

    if (seconds >= 2592000) {
        let months = ~~(seconds / 2592000)
        seconds = seconds - months * 2592000
        out += months + 'M '
    }

    if (seconds >= 86400) {
        let days = ~~(seconds / 86400)
        seconds = seconds - days * 86400
        out += days + 'd '
    }

    if (seconds >= 3600) {
        let hours = ~~(seconds / 3600)
        seconds = seconds - hours * 3600
        out += hours + 'h '
    }

    if (seconds >= 60) {
        let minutes = ~~(seconds / 60)
        seconds = seconds - minutes * 60
        out += minutes + 'm '
    }

    if (seconds > 0) {
        out += seconds + 's '
    }

    return out + 'ago'
}
