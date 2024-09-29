import { SiteModel } from 'models'
import { fmt_timeago, httpx } from 'shared'
import { createEffect, onMount, Show } from 'solid-js'
import { createStore, produce } from 'solid-js/store'

import './style/dash.scss'

const LOOP_TIMEOUT = 2e3
const SOCKET_STATUS = {
    offline: ['Offline', 'var(--mc-3)', 'var(--green)'],
    online: ['Online', 'var(--green)', 'var(--red)'],
} as const

export default () => {
    type State = {
        sites: { [id: string]: SiteModel }
        socket: WebSocket | null
        socket_status: keyof typeof SOCKET_STATUS
        loop: number | null
    }
    const [state, setState] = createStore<State>({
        sites: {},
        socket: null,
        socket_status: 'offline',
        loop: null,
    })

    onMount(() => load(0))

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
                        })
                    })
                )
                if (x.response.length == 32) return load(page + 1)
            },
        })
    }

    function connect() {
        let host =
            location.protocol == 'http:'
                ? 'ws://localhost:7000'
                : `wss://${location.host}`
        let socket = new WebSocket(`${host}/api/sites/ws-test/`)
        setState({ socket })
    }

    createEffect(() => {
        if (state.socket == null) {
            setState({ socket_status: 'offline' })
            return
        }

        function onclose() {
            setState(
                produce(s => {
                    s.socket = null
                    s.socket_status = 'offline'
                    clearInterval(s.loop)
                    s.loop = null
                })
            )
        }

        state.socket.onopen = () => {
            if (state.loop != null) clearInterval(state.loop)

            setState({
                socket_status: 'online',
                loop: setInterval(() => {
                    if (state.socket == null) {
                        clearInterval(state.loop)
                        return
                    }
                    Object.keys(state.sites).map(id => state.socket.send(id))
                }, LOOP_TIMEOUT),
            })
        }
        state.socket.onclose = onclose
        state.socket.onerror = onclose

        state.socket.onmessage = e => {
            setState(
                produce(s => {
                    let site = JSON.parse(e.data) as SiteModel
                    if (!site.id) return alert('err')
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
                            setState({ socket: null })
                        } else {
                            connect()
                        }
                    }}
                >
                    {SOCKET_STATUS[state.socket_status][0]}
                </button>
            </div>
            <div class='site-list'>
                {Object.values(state.sites).map(site => (
                    <div class='site'>
                        <div class='site-info'>
                            <span>id | name:</span>
                            <span>
                                {site.id} | {site.name}
                            </span>
                            <span>latest request:</span>
                            <span>{fmt_timeago(site.latest_request)}</span>
                            <span>latest ping:</span>
                            <span>{fmt_timeago(site.latest_ping)}</span>
                            <span>total requests:</span>
                            <span>{site.total_requests.toLocaleString()}</span>
                            <span>average request time:</span>
                            <span>
                                <Show
                                    when={
                                        site.total_requests != 0 &&
                                        site.total_requests_time != 0
                                    }
                                    fallback={'0s'}
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
                        </div>
                        <div class='line' />
                        <div class='site-status'>
                            {Object.entries(site.status).map(
                                ([status, count]) => (
                                    <>
                                        <span>{status}:</span>
                                        <span>{count.toLocaleString()}</span>
                                    </>
                                )
                            )}
                        </div>
                    </div>
                ))}
            </div>
        </div>
    )
}
