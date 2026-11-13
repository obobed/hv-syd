import { Link, useLocation, useNavigate } from 'react-router'
import logo from '../assets/hvsydLogoTrans.svg'
import flag from '../assets/haven/hack-club-flag.png'
import avatar from '../assets/haven/hedgehog.webp'
import { clearSession } from '../lib/session'
import { readStaffSection, staffSections } from '../lib/staff-sections'
import './navbar.css'

export default function Navbar({ session }) {
    const navigate = useNavigate()
    const { search } = useLocation()
    const isStaff = session?.role === 'staff'
    const section = isStaff ? readStaffSection(search) : null

    function signOut() {
        clearSession()
        navigate('/', { replace: true })
    }

    return (
        <nav className={isStaff ? 'navbg navbg-staff' : 'navbg'} aria-label="Main navigation">
            <Link to="/dashboard" className="haven-nav-brand" aria-label="Haven Sydney dashboard">
                <img src={flag} alt="" className="haven-nav-flag" width="526" height="184" />
                <img src={logo} alt="" className="haven-nav-logo" width="1402" height="1402" />
            </Link>
            {isStaff && (
                <div className="haven-nav-links" role="group" aria-label="Staff dashboard sections">
                    {staffSections.map(({ id, label, to }) => (
                        <Link key={id} to={to} aria-current={section === id ? 'page' : undefined}>{label}</Link>
                    ))}
                </div>
            )}
            {session ? (
                <div className="haven-nav-account">
                    <span className="haven-nav-avatar">
                        <img src={avatar} alt="" />
                    </span>
                    <span className="haven-nav-name" title={session.name}>{session.name}</span>
                    <button className="haven-nav-action haven-nav-signout" type="button" aria-label="Sign out" onClick={signOut}>
                        <svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="none"
                            stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M10 4H5a1 1 0 0 0-1 1v14a1 1 0 0 0 1 1h5m-1-8h11m-4-4 4 4-4 4" />
                        </svg>
                        <span>Sign out</span>
                    </button>
                </div>
            ) : session === null && (
                <Link className="haven-nav-action haven-nav-signin" to="/">Sign in</Link>
            )}
        </nav>
    )
}
