import { ChevronDownIcon, ChevronUpIcon } from 'icons'
import './style/select.scss'
import { createStore, produce } from 'solid-js/store'
import { Component, JSXElement, Show, createEffect, on } from 'solid-js'

type Props<T extends string | number> = {
    items: [T, JSXElement][]
    onChange(props: T[]): void
    selected: T[]
    multiple?: true | number
    disabled?: boolean
    placeholder?: string
}

export const Select = <T extends string | number>(P: Props<T>) => {
    type State = {
        open: boolean
        selected: number[]
        changed: number
        ph: string
    }
    const [state, setState] = createStore<State>({
        open: false,
        selected: P.selected.map(id => P.items.findIndex(i => i[0] == id)),
        changed: 0,
        ph: P.placeholder || '---',
    })

    createEffect(() => {
        setState({
            selected: P.selected.map(id => P.items.findIndex(i => i[0] == id)),
        })
    })

    createEffect(
        on(
            () => state.changed,
            () => P.onChange(state.selected.map(idx => P.items[idx][0])),
            { defer: true }
        )
    )

    createEffect(() => {
        if (P.disabled) setState({ open: false })
    })

    return (
        <div class='cmp-select' classList={{ disabled: P.disabled }}>
            <div
                class='cmp-select-head'
                onclick={() =>
                    !P.disabled && setState(s => ({ open: !s.open }))
                }
            >
                <Show
                    when={P.multiple}
                    fallback={
                        <>
                            {(P.items[state.selected[0]] &&
                                P.items[state.selected[0]][1]) ||
                                state.ph}
                        </>
                    }
                >
                    <div class='selected'>
                        {state.selected.map(item => (
                            <div class='item'>{P.items[item][1]}</div>
                        ))}
                        {!state.selected.length && state.ph}
                    </div>
                </Show>
                <Show when={!P.disabled}>
                    {state.open ? <ChevronUpIcon /> : <ChevronDownIcon />}
                </Show>
            </div>
            <div
                class='cmp-select-body'
                classList={{ active: state.open, multiple: !!P.multiple }}
            >
                {P.items.map(([id, item], idx) => (
                    <div
                        class='item'
                        classList={{
                            active: state.selected.includes(idx),
                        }}
                        onclick={() =>
                            setState(
                                produce(s => {
                                    if (!P.multiple) {
                                        s.selected = [idx]
                                        s.changed = performance.now()
                                        return
                                    }

                                    let x = s.selected.indexOf(idx)

                                    if (x != -1) {
                                        s.selected.splice(x, 1)
                                    } else {
                                        if (
                                            typeof P.multiple == 'number' &&
                                            s.selected.length >= P.multiple
                                        )
                                            return

                                        s.selected.push(idx)
                                    }

                                    s.changed = performance.now()
                                })
                            )
                        }
                    >
                        {item}
                    </div>
                ))}
            </div>
        </div>
    )
}
