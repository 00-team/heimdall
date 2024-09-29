import { httpx } from 'shared'
import { createEffect, createRoot, on } from 'solid-js'
import { createStore } from 'solid-js/store'

type State = {
    update: number
    data: { [k: string]: [number, number] }
}

const [prices, setPrices] = createStore<State>({
    update: 0,
    data: {},
})

createRoot(() => {
    createEffect(
        on(
            () => {
                prices.update
            },
            () => {
                if (prices.update + 600e3 > Date.now()) return

                httpx({
                    method: 'GET',
                    url: '/api/vendor/prices/',
                    onLoad(x) {
                        if (x.status != 200) return

                        let country: State['data'] = {}
                        let service: State['data'] = {}
                        let data: State['data'] = x.response
                        Object.entries(data).map(([k, [cost]]) => {
                            let [c, s] = k.split('-')
                            if (c in country) {
                                country[c][0] += cost
                                country[c][1] += 1
                            } else {
                                country[c] = [cost, 1]
                            }

                            if (s in service) {
                                service[s][0] += cost
                                service[s][1] += 1
                            } else {
                                service[s] = [cost, 1]
                            }
                        })

                        setPrices({
                            update: Date.now(),
                            data: { ...data, ...country, ...service },
                        })
                    },
                })
            },
            { defer: true }
        )
    )
})

export { prices, setPrices }
