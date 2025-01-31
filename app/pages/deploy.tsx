import { A, useSearchParams } from '@solidjs/router'
import { Deploy } from 'models'
import {
    fmt_duration,
    fmt_timeago,
    fmt_timeago_ts,
    fmt_timestamp,
    httpx,
} from 'shared'
import { Component, createSignal, onCleanup, onMount, Show } from 'solid-js'
import { createStore } from 'solid-js/store'

import './style/deploy.scss'
import { ChevronLeftIcon, ChevronRightIcon } from 'icons'
const DIV_TS = new Date().getTime() / 1e3 - 30 * 24 * 3600

export default () => {
    type State = {
        deploys: Deploy[]
        page: number
        loading: boolean
    }
    const [state, setState] = createStore<State>({
        deploys: [],
        page: 0,
        loading: true,
    })

    const [search_params, setSearchParams] = useSearchParams()

    onMount(() => {
        let page = parseInt(search_params.page) || 0
        load(page)
    })

    function load(page: number) {
        setState({ loading: true })
        httpx({
            url: '/api/deploy/',
            method: 'GET',
            params: { page },
            onLoad(x) {
                if (x.status != 200) return
                setState({ deploys: x.response, page, loading: false })
                setSearchParams({ page })
            },
        })
    }

    return (
        <div class='deploy-fnd'>
            <div class='navbar'>
                <div>
                    <A href='/'>home</A>
                </div>
                <div class='pagination'>
                    <button
                        class='styled icon'
                        disabled={state.page < 1}
                        onClick={() => load(state.page - 1)}
                    >
                        <ChevronLeftIcon />
                    </button>
                    <button
                        class='styled icon'
                        disabled={state.deploys.length != 32}
                        onClick={() => load(state.page + 1)}
                    >
                        <ChevronRightIcon />
                    </button>
                </div>
            </div>
            <div class='deploy-list'>
                <Show when={state.loading}>
                    <div class='message-fnd'>
                        <div class='message'>Loading</div>
                    </div>
                </Show>
                <Show when={state.deploys.length == 0 && !state.loading}>
                    <div class='message-fnd'>
                        <div class='message'>Empty</div>
                    </div>
                </Show>

                {state.deploys.map(d => (
                    <div class='deploy' classList={{ [d.status]: true }}>
                        <div class='row'>
                            <span class='id'>{d.id}</span>
                            <span class='space'>|</span>
                            <span class='status'>{d.status}</span>
                        </div>
                        <div class='row'>
                            {d.sender}
                            <span class='space'>@</span>
                            {d.actor}
                            <span class='space'>in</span> {d.repo}
                        </div>
                        <div class='row'>
                            <Show
                                when={DIV_TS > d.begin}
                                fallback={
                                    <span>{fmt_timeago_ts(d.begin)}</span>
                                }
                            >
                                <span>{fmt_timestamp(d.begin)}</span>
                            </Show>
                            <span class='space'>|</span>
                            <Show when={d.finish} fallback={'never finished'}>
                                <span>{fmt_duration(d.finish - d.begin)}</span>
                            </Show>
                        </div>
                        <Std std={d.stdout} title='stdout' />
                        <Std std={d.stderr} title='stderr' />
                    </div>
                ))}
            </div>
        </div>
    )
}

const Std: Component<{ std: string | null; title: string }> = P => {
    let ref: HTMLDivElement
    const [is_fullscrenn, setFullScreen] = createSignal(false)

    function fschange() {
        setFullScreen(ref && document.fullscreenElement == ref)
    }

    function toggle_fs() {
        if (document.fullscreenElement == ref) {
            document.exitFullscreen()
        } else {
            ref.requestFullscreen()
        }
    }

    onMount(() => {
        document.addEventListener('fullscreenchange', fschange)
    })

    onCleanup(() => {
        document.removeEventListener('fullscreenchange', fschange)
    })

    return (
        <Show when={P.std}>
            <div
                class='std'
                classList={{ fullscreen: is_fullscrenn() }}
                ref={ref}
            >
                <span class='title' onclick={toggle_fs}>
                    {P.title}
                </span>
                <textarea class='styled' value={P.std}></textarea>
            </div>
        </Show>
    )
}
