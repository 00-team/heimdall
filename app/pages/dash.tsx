import { SiteMessageModel, SiteModel } from 'models'
import { fmt_timeago, fmt_timestamp, httpx } from 'shared'
import { onMount, Show } from 'solid-js'
import { createStore, produce } from 'solid-js/store'

import './style/dash.scss'
type LineStatus = {
    name: string
    color: string
    hover: string
}

const DEFAULT_INTERVAL = 10
const LONG_INTERVAL = 1000
const LINE_STATUS: { [k in 'offline' | 'online']: LineStatus } = {
    offline: { name: 'Offline', color: 'var(--mc-3)', hover: 'var(--green)' },
    online: { name: 'Online', color: 'var(--green)', hover: 'var(--red)' },
}

const ORDER: number[] = JSON.parse(localStorage.getItem('order') || '[]') || []

function order_get(id: number) {
    for (let i = 0; i < ORDER.length; i++) {
        if (ORDER[i] == id) return i
    }
    return 999
}

export default () => {
    type State = {
        sites: { [id: string]: SiteModel }
        messages: { [id: string]: SiteMessageModel[] }
        timer: number
        interval: number
        online: boolean
        status: LineStatus
        now: number
        act(): void
    }
    const [state, setState] = createStore<State>({
        sites: {},
        messages: {},
        online: false,
        get status() {
            return this.online ? LINE_STATUS.online : LINE_STATUS.offline
        },
        timer: DEFAULT_INTERVAL,
        interval: DEFAULT_INTERVAL,
        now: 0,
        act() {
            load(0)
        },
    })

    onMount(() => {
        window.onfocus = () => {
            setState({ timer: 0, interval: DEFAULT_INTERVAL })
        }
        window.onblur = () => setState({ interval: LONG_INTERVAL })

        setState(
            produce(s => {
                setInterval(() => {
                    s.now = ~~(new Date().getTime() / 1e3)
                    if (!s.online) return
                    if (s.timer <= 0) {
                        s.act()
                        s.timer = s.interval
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
                            if (site.id in s.sites) {
                                let old = s.sites[site.id]
                                if (
                                    old.latest_message_timestamp !=
                                    site.latest_message_timestamp
                                ) {
                                    load_messages(site.id)
                                }
                            } else {
                                s.sites[site.id] = site
                                load_messages(site.id)
                            }
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

    return (
        <div class='dash-fnd'>
            <div class='status-bar'>
                <button
                    class='styled connection'
                    style={{
                        '--bd': state.status.color,
                        '--hv-bd': state.status.hover,
                    }}
                    onClick={() => setState(s => ({ online: !s.online }))}
                >
                    {state.status.name}
                </button>
                <span>{state.timer}s</span>
            </div>
            <div class='site-list'>
                {Object.values(state.sites)
                    .sort((a, b) => order_get(a.id) - order_get(b.id))
                    .map(site => (
                        <div
                            class='site'
                            classList={{
                                offline: state.now - site.latest_ping > 120,
                            }}
                        >
                            <div class='site-info'>
                                <span>id | name:</span>
                                <span>
                                    {site.id} | {site.name} |{' '}
                                    {site.online ? '✅' : '�'}
                                </span>
                                <span>from:</span>
                                <span>{fmt_timestamp(site.timestamp)}</span>
                                <span>request / ping:</span>
                                <div class='with-space'>
                                    <span>
                                        {fmt_timeago(
                                            state.now - site.latest_request
                                        )}
                                    </span>
                                    <span class='spacer'>|</span>
                                    <span>
                                        {fmt_timeago(
                                            state.now - site.latest_ping
                                        )}
                                    </span>
                                </div>
                                <span>time / count:</span>
                                <div class='with-space'>
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
                                    <span class='spacer'>/</span>
                                    <span>
                                        {site.total_requests.toLocaleString()}
                                    </span>
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
                                            <td>
                                                {~~(s.total_time / s.count)}ms
                                            </td>
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
                                                            {i <
                                                                a.length -
                                                                    1 && <br />}
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
