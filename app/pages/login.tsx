import { createStore } from 'solid-js/store'
import './style/login.scss'
import { httpx } from 'shared'
import { Show } from 'solid-js'
import { setSelf } from 'store'

export default () => {
    type State = {
        phone: string
        code: string
        stage: 'code' | 'phone'
    }
    const [state, setState] = createStore<State>({
        phone: '',
        code: '',
        stage: 'phone',
    })

    function verification() {
        httpx({
            url: '/api/verification/',
            method: 'POST',
            json: {
                action: 'login',
                phone: state.phone,
            },
            onLoad(x) {
                if (x.status != 200) return

                setState({ stage: 'code' })
            },
        })
    }

    function login() {
        httpx({
            url: '/api/user/login/',
            method: 'POST',
            json: {
                code: state.code,
                phone: state.phone,
            },
            onLoad(x) {
                if (x.status != 200) return
                setSelf({ user: x.response, loged_in: true })
            },
        })
    }

    return (
        <div class='login-fnd'>
            <div class='login-form'>
                <h1>Heimdall</h1>
                <span>
                    {state.stage == 'phone'
                        ? 'enter your phone'
                        : 'enter verification code'}
                </span>
                <Show
                    when={state.stage == 'phone'}
                    fallback={
                        <input
                            class='styled'
                            placeholder='code: e.g. 12345'
                            value={state.code}
                            onInput={e => {
                                setState({ code: e.currentTarget.value })
                            }}
                        />
                    }
                >
                    <input
                        class='styled'
                        placeholder='phone: e.g. 09182325555'
                        value={state.phone}
                        autocomplete='phone'
                        onInput={e => {
                            setState({ phone: e.currentTarget.value })
                        }}
                    />
                </Show>
                <Show
                    when={state.stage == 'phone'}
                    fallback={
                        <button class='styled' onclick={login}>
                            Verify Code
                        </button>
                    }
                >
                    <button class='styled' onclick={verification}>
                        Send Code
                    </button>
                </Show>
            </div>
        </div>
    )
}
