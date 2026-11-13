import { Link, NavLink, useNavigate } from 'react-router'
import logo from '../assets/hvsydLogoTrans.svg'
import { clearSession } from '../lib/session'
import avatar from '../assets/haven/hedgehog.webp'
import './navbar.css'

export default function Navbar({ session }) {
    const navigate = useNavigate()

    function signOut() {
        clearSession()
        navigate('/', { replace: true })
    }

    return (
        <nav className="navbg" aria-label="Main navigation">
            <Link to="/dashboard" className="haven-nav-brand" aria-label="Haven Sydney dashboard">
                <img src={logo} alt="" className="logo" />
            </Link>
            <div className="haven-nav-links">
                <NavLink to="/dashboard">Dashboard</NavLink>
                {session?.role === 'staff' && <NavLink to="/rollcall">Rollcall</NavLink>}
            </div>
            {session ? (
                <div className="haven-nav-account">
                    <span className="haven-nav-avatar">
                        <img src={avatar} alt="" />
                    </span>
                    <span className="haven-nav-name" title={session.name}>{session.name}</span>
                    <button className="haven-nav-action" type="button" onClick={signOut}>
                        <svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="none"
                            stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M10 4H5a1 1 0 0 0-1 1v14a1 1 0 0 0 1 1h5m-1-8h11m-4-4 4 4-4 4" />
                        </svg>
                        Sign out
                    </button>
                </div>
            ) : session === null && (
                <Link className="haven-nav-action haven-nav-signin" to="/">Sign in</Link>
            )}
        </nav>
    )
}
