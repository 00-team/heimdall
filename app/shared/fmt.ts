export function fmt_timestamp(ts: number): string {
    let dt = new Date(ts * 1e3)

    let date = `${dt.getFullYear()}/${dt.getMonth()}/${dt.getDate()}`
    let time = `${dt.getHours()}:${dt.getMinutes()}:${dt.getSeconds()}`

    return date + ' - ' + time
}

export function fmt_timeago(ts: number): string {
    if (ts == 0) return '---'
    let now = ~~(new Date().getTime() / 1e3)
    return `${now - ts}s ago`
}
