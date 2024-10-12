export type SiteStatusModel = {
    code: number
    count: number
    max_time: number
    min_time: number
    total_time: number
}

export type SiteModel = {
    id: number
    name: string
    timestamp: number
    latest_request: number
    latest_ping: number
    total_requests: number
    total_requests_time: number
    requests_max_time: number
    requests_min_time: number
    status: { [k: string]: SiteStatusModel }
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
