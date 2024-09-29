export * from './httpx'
export * from './fmt'

export function now() {
    return ~~(new Date().getTime() / 1e3)
}
