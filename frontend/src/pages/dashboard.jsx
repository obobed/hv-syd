import { Link, useLocation, useOutletContext } from 'react-router'
import { readStaffSection } from '../lib/staff-sections'
import Rollcall from './rollcall'
import StaffSubmissions from './staff-submissions'
import StaffTokens from './staff-tokens'
import TeamDashboard from './team-dashboard'
import './staff-dashboard.css'

export default function Dashboard() {
    const session = useOutletContext()
    if (session === undefined) return <p role="status">Checking your session…</p>
    if (!session) return <p><Link to="/">Sign in to view your dashboard</Link></p>
    if (session.role === 'staff') return <StaffDashboard />
    return <TeamDashboard />
}

function StaffDashboard() {
    const { search } = useLocation()
    const section = readStaffSection(search)
    return (
        <main className="haven-team haven-staff" aria-label="Staff dashboard">
            <div className="haven-team-shell haven-staff-shell">
                {section === 'review' && <StaffSubmissions />}
                {section === 'roster' && <Rollcall />}
                {section === 'access' && <StaffTokens />}
            </div>
        </main>
    )
}
