import { SiteMessageModel, SiteModel } from 'models'
import { fmt_timeago, fmt_timestamp, httpx } from 'shared'
import { createEffect, onMount, Show } from 'solid-js'
import { createStore, produce } from 'solid-js/store'

import './style/dash.scss'

const SOCKET_STATUS = {
    offline: ['Offline', 'var(--mc-3)', 'var(--green)'],
    online: ['Online', 'var(--green)', 'var(--red)'],
} as const

export default () => {
    type State = {
        sites: { [id: string]: SiteModel }
        messages: { [id: string]: SiteMessageModel[] }
        socket: WebSocket | null
        socket_status: keyof typeof SOCKET_STATUS
        timer: number
        online: boolean
        now: number
        focus: boolean
        act(): void
    }
    const [state, setState] = createStore<State>({
        sites: {},
        messages: {},
        socket: null,
        socket_status: 'offline',
        online: false,
        timer: 5,
        focus: true,
        now: 0,
        act() {
            if (!this.focus) return
            if (this.online) {
                if (!this.socket || this.socket.readyState != WebSocket.OPEN) {
                    return connect()
                }
            } else return

            Object.values(state.sites)
                .filter(site => site.online)
                .forEach(site => this.socket.send(site.id.toString()))
        },
    })

    onMount(() => {
        window.onfocus = () => setState({ focus: true })
        window.onblur = () => setState({ focus: false })

        setState(
            produce(s => {
                setInterval(() => {
                    s.now = ~~(new Date().getTime() / 1e3)
                    if (!s.socket || s.socket.readyState != WebSocket.OPEN)
                        return
                    if (s.timer <= 0) {
                        s.act()
                        s.timer = 5
                    } else s.timer--
                }, 1e3)
            })
        )
        load(0)
    })

    function load(page: number) {
        httpx({
            url: '/api/sites/',
            method: 'GET',
            params: { page },
            onLoad(x) {
                if (x.status != 200 || x.response.length == 0) return
                let sites = x.response as SiteModel[]
                setState(
                    produce(s => {
                        sites.forEach(site => {
                            s.sites[site.id] = site
                            load_messages(site.id)
                        })
                    })
                )
                if (x.response.length == 32) return load(page + 1)
            },
        })
    }

    function load_messages(site_id: number) {
        httpx({
            url: `/api/sites/${site_id}/messages/`,
            method: 'GET',
            onLoad(x) {
                if (x.status != 200 || x.response.length == 0) return
                let messages = x.response as SiteMessageModel[]
                setState(
                    produce(s => {
                        s.messages[site_id] = messages.slice(0, 3)
                    })
                )
            },
        })
    }

    function connect() {
        let host =
            location.protocol == 'http:'
                ? 'ws://localhost:7000'
                : `wss://${location.host}`
        let socket = new WebSocket(`${host}/api/sites/live/`)
        setState({ socket })
    }

    createEffect(() => {
        if (state.socket == null) {
            setState({ socket_status: 'offline' })
            return
        }

        function onclose() {
            setState({ socket: null, socket_status: 'offline' })
        }

        state.socket.onopen = () => {
            setState({ socket_status: 'online' })
        }
        state.socket.onclose = onclose
        state.socket.onerror = onclose

        state.socket.onmessage = e => {
            setState(
                produce(s => {
                    let site = JSON.parse(e.data) as SiteModel
                    if (!site.id) return alert('err')
                    if (
                        s.sites[site.id].latest_message_timestamp !=
                        site.latest_message_timestamp
                    ) {
                        load_messages(site.id)
                    }
                    s.sites[site.id] = site
                })
            )
        }
    })

    return (
        <div class='dash-fnd'>
            <div class='status-bar'>
                <button
                    class='styled connection'
                    style={{
                        '--bd': SOCKET_STATUS[state.socket_status][1],
                        '--hv-bd': SOCKET_STATUS[state.socket_status][2],
                    }}
                    onClick={() => {
                        if (
                            state.socket &&
                            state.socket.readyState == WebSocket.OPEN
                        ) {
                            state.socket.close()
                            setState({ socket: null, online: false })
                        } else {
                            setState({ online: true })
                            connect()
                        }
                    }}
                >
                    {SOCKET_STATUS[state.socket_status][0]}
                </button>
                <span>{state.timer}s</span>
            </div>
            <div class='site-list'>
                {Object.values(state.sites).map(site => (
                    <div
                        class='site'
                        classList={{
                            offline: state.now - site.latest_ping > 120,
                        }}
                    >
                        <div class='site-info'>
                            <span>id | name:</span>
                            <span>
                                {site.id} | {site.name}
                            </span>
                            <span>online:</span>
                            <span>{site.online ? '✅' : '❌'}</span>
                            <span>date:</span>
                            <span>{fmt_timestamp(site.timestamp)}</span>
                            <span>latest request:</span>
                            <span>
                                {fmt_timeago(state.now - site.latest_request)}
                            </span>
                            <span>latest ping:</span>
                            <span>
                                {fmt_timeago(state.now - site.latest_ping)}
                            </span>
                            <span>count:</span>
                            <span>{site.total_requests.toLocaleString()}</span>
                            <span>time:</span>
                            <div class='request-time'>
                                <span>{site.requests_min_time}ms</span>
                                <span class='spacer'>|</span>
                                <span>
                                    <Show
                                        when={
                                            site.total_requests != 0 &&
                                            site.total_requests_time != 0
                                        }
                                        fallback={'0ms'}
                                    >
                                        {
                                            ~~(
                                                site.total_requests_time /
                                                site.total_requests
                                            )
                                        }
                                        ms
                                    </Show>
                                </span>
                                <span class='spacer'>|</span>
                                <span>{site.requests_max_time}ms</span>
                            </div>
                        </div>
                        <div class='line' />
                        <table class='site-status-table'>
                            <thead>
                                <tr>
                                    <th>code</th>
                                    <th>min</th>
                                    <th>avg</th>
                                    <th>max</th>
                                    <th>count</th>
                                </tr>
                            </thead>
                            <tbody>
                                {Object.values(site.status).map(s => (
                                    <tr>
                                        <td>{s.code}</td>
                                        <td>{s.min_time}ms</td>
                                        <td>{~~(s.total_time / s.count)}ms</td>
                                        <td>{s.max_time}ms</td>
                                        <td>{s.count}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                        <Show when={state.messages[site.id]}>
                            <div class='line' />
                            <div class='site-messages'>
                                {state.messages[site.id].map(msg => (
                                    <div class='message'>
                                        <span class='tag'>{msg.tag}</span>
                                        <p>
                                            {msg.text
                                                .split('\n')
                                                .map((l, i, a) => (
                                                    <>
                                                        {l}
                                                        {i < a.length - 1 && (
                                                            <br />
                                                        )}
                                                    </>
                                                ))}
                                        </p>
                                        <span class='timestamp'>
                                            {fmt_timeago(
                                                state.now - msg.timestamp
                                            )}
                                        </span>
                                    </div>
                                ))}
                            </div>
                        </Show>
                        {/*<div class='line' />
                        <div class='site-actions'>
                            <button class='styled'>Site</button>
                        </div>*/}
                    </div>
                ))}
            </div>
        </div>
    )
}
