export type SiteModel = {
    id: number
    name: string
    latest_request: number
    latest_ping: number
    total_requests: number
    total_requests_time: number
    status: { [k: string]: number }
    token: string | null
    online: boolean
    latest_message_timestamp: number
}

export type SiteMessageModel = {
    id: number
    site: number
    timestamp: number
    text: string
    tag: string
}

export type UserModel = {
    id: number
    phone: string
    name: string
    token: string | null
    admin: boolean
}
