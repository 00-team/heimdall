import { SiteModel } from 'models'
import { httpx } from 'shared'
import { onMount } from 'solid-js'
import { createStore, produce } from 'solid-js/store'

import './style/dash.scss'

export default () => {
    type State = {
        sites: { [id: string]: SiteModel }
    }
    const [state, setState] = createStore<State>({
        sites: {},
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

    return (
        <div class='dash-fnd'>
            <div class='status-bar'>
                <button class='styled connection'>Connecting ...</button>
            </div>
            <div class='site-list'>
                {Object.values(state.sites).map(site => (
                    <div class='site'>
                        <span>id</span>
                        <span>{site.id}</span>
                        <span>name</span>
                        <span>{site.name}</span>
                        <span>latest_request</span>
                        <span>{site.latest_request}</span>
                        <span>latest_ping</span>
                        <span>{site.latest_ping}</span>
                        <span>total_requests</span>
                        <span>{site.total_requests}</span>
                        <span>total_requests_time</span>
                        <span>{site.total_requests_time}</span>
                        <span>token</span>
                        <input
                            class='styled'
                            value={site.token}
                            onInput={e => (e.currentTarget.value = site.token)}
                        />
                        <div class='site-status'>
                            {Object.entries(site.status).map(
                                ([status, count]) => (
                                    <>
                                        <span>{status}:</span>
                                        <span>{count}</span>
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
