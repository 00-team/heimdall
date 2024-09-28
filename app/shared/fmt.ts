export function fmt_timestamp(ts: number): string {
    let dt = new Date(ts * 1e3)

    let date = `${dt.getFullYear()}/${dt.getMonth()}/${dt.getDate()}`
    let time = `${dt.getHours()}:${dt.getMinutes()}:${dt.getSeconds()}`

    return date + ' - ' + time
}
