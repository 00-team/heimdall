import './style/login.scss'

export default () => {
    return (
        <div class='login-fnd'>
            <div class='login-form'>
                <h1>Heimdall</h1>
                <span>XXX</span>
                <button
                    class='styled'
                    onclick={() => open('https://t.me/Thorabot?start=login')}
                >
                    Login
                </button>
            </div>
        </div>
    )
}
