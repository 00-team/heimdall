export type SiteModel = {
    id: number
    name: string
    latest_request: number
    latest_ping: number
    total_requests: number
    total_requests_time: number
    status: { [k: string]: number }
    token: string | null
}
