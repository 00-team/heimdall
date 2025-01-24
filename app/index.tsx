import { Show, render } from 'solid-js/web'
import { self } from 'store'
import 'style/index.scss'
import Login from 'pages/login'
import Dash from 'pages/dash'
import Deploy from 'pages/deploy'
import Alert from 'comps/alert'
import { Route, Router } from '@solidjs/router'
import { lazy } from 'solid-js'

const Root = () => {
    return (
        <>
            <Show when={self.loged_in} fallback={<Login />}>
                <Router>
                    <Route path='/' component={Dash} />
                    <Route path='/deploy/' component={Deploy} />
                    <Route
                        path='*path'
                        component={lazy(() => import('pages/404'))}
                    />
                </Router>
            </Show>
            <Alert />
        </>
    )
}

render(Root, document.getElementById('root'))
