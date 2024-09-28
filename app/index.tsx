import { render } from 'solid-js/web'

import './style/index.scss'
import './style/sites.scss'
import { onMount } from 'solid-js'
import { createStore, produce } from 'solid-js/store'
import { SiteModel } from 'models'
import { fmt_timestamp } from 'shared/fmt'

const Root = () => {
    type State = {
        sites: { [k: string]: SiteModel }
    }
    const [state, setState] = createStore<State>({
        sites: {},
    })

    onMount(() => {
        const proto = location.protocol.startsWith('https') ? 'wss' : 'ws'
        const ws_uri = `${proto}://localhost:7000/api/sites/ws-test/`
        let socket = new WebSocket(ws_uri)
        console.log(ws_uri)

        socket.onopen = () => {
            console.log('connected')
        }

        socket.onmessage = e => {
            try {
                let data = JSON.parse(e.data)
                setState(
                    produce(s => {
                        s.sites[data.id] = data
                        s.sites[3] = data
                        s.sites[5] = data
                        s.sites[12] = data
                    })
                )
            } catch {}
        }

        socket.onclose = () => {
            console.log('closed')
        }

        setInterval(() => {
            if (socket.readyState == socket.OPEN) {
                socket.send('1')
            }
        }, 1000)
    })

    return (
        <div class='site-list'>
            {Object.values(state.sites).map(site => (
                <div class='site'>
                    <span>id:</span>
                    <span>{site.id}</span>
                    <span>name:</span>
                    <span>{site.name}</span>
                    <span>latest request:</span>
                    <span>{fmt_timestamp(site.latest_request)}</span>
                    <span>latest ping:</span>
                    <span>{fmt_timestamp(site.latest_ping)}</span>
                    <span>total requests:</span>
                    <span>{site.total_requests}</span>
                    <span>total requests time:</span>
                    <span>{site.total_requests_time}</span>
                </div>
            ))}
        </div>
    )
}

render(Root, document.getElementById('root')!)
