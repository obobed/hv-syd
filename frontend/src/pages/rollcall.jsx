import { useEffect, useState } from 'react'
import './rollcall.css'

const checkInTime = new Intl.DateTimeFormat(undefined, { hour: 'numeric', minute: '2-digit' })

function attendeeName(attendee) {
    return [attendee.first_name, attendee.last_name].filter(Boolean).join(' ') || 'Unnamed attendee'
}

async function readRoster(signal) {
    const response = await fetch('/api/admin/rollcall', {
        credentials: 'same-origin', cache: 'no-store', redirect: 'error',
        signal: AbortSignal.any([signal, AbortSignal.timeout(10000)]),
    })
    if (!response.ok || !response.headers.get('content-type')?.includes('application/json')) {
        throw new Error('Couldn’t load the roster.')
    }
    const roster = await response.json()
    if (!Array.isArray(roster?.attendees)) throw new Error('Couldn’t load the roster.')
    return roster
}

function AttendeeList({ attendees, label, attendanceAvailable, showTeam = true }) {
    return (
        <ul className="haven-rollcall-list" aria-label={label}>
            {attendees.map(attendee => (
                <li key={attendee.id} className="haven-rollcall-attendee">
                    <div>
                        <strong>{attendeeName(attendee)}</strong>
                        {showTeam && attendee.team_name && <p>{attendee.team_name}</p>}
                    </div>
                    {attendee.checked_in_at ? (
                        <span className="haven-rollcall-present">
                            <svg aria-hidden="true" width="16" height="16" viewBox="0 0 24 24" fill="none"
                                stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round">
                                <path d="m5 12 4 4L19 6" />
                            </svg>
                            <span className="haven-visually-hidden">Checked in at </span>
                            <time dateTime={attendee.checked_in_at}>{checkInTime.format(new Date(attendee.checked_in_at))}</time>
                        </span>
                    ) : attendanceAvailable && (
                        <span className="haven-rollcall-status">{attendee.attend_registration_found ? 'Awaiting check-in' : 'No matching Attend record'}</span>
                    )}
                </li>
            ))}
        </ul>
    )
}

export default function Rollcall() {
    const [roster, setRoster] = useState(null)
    const [loadError, setLoadError] = useState(false)
    const [attempt, setAttempt] = useState(0)
    const [query, setQuery] = useState('')
    const [view, setView] = useState('attendees')

    useEffect(() => {
        const controller = new AbortController()
        readRoster(controller.signal).then(result => {
            if (!controller.signal.aborted) setRoster(result)
        }).catch(() => {
            if (!controller.signal.aborted) setLoadError(true)
        })
        return () => controller.abort()
    }, [attempt])

    const attendees = roster?.attendees ?? []
    const checkedIn = attendees.filter(attendee => attendee.checked_in_at).length
    const search = query.trim().toLowerCase()
    const visible = attendees.filter(attendee =>
        [attendeeName(attendee), attendee.team_name, attendee.id].some(value => value?.toLowerCase().includes(search)))

    const teams = new Map()
    if (view === 'teams') {
        for (const attendee of visible) {
            const name = attendee.team_name
            if (!teams.has(name)) teams.set(name, [])
            teams.get(name).push(attendee)
        }
    }

    return (
        <section className="haven-team-card haven-rollcall-roster" aria-labelledby="rollcall-roster-title">
            <header className="haven-staff-heading">
                <h2 id="rollcall-roster-title">Roster</h2>
                {roster && (
                    <span className="haven-rollcall-total">
                        {roster.attendance_available
                            ? `${checkedIn} of ${attendees.filter(attendee => attendee.attend_registration_found).length} checked in`
                            : `${attendees.length} ${attendees.length === 1 ? 'attendee' : 'attendees'}`}
                    </span>
                )}
            </header>
            {!roster && !loadError && <p role="status">Loading roster…</p>}
            {loadError && (
                <div className="haven-staff-actions">
                    <p className="haven-team-error" role="alert">Couldn’t load the roster.</p>
                    <button type="button" className="haven-team-secondary" onClick={() => {
                        setLoadError(false)
                        setAttempt(current => current + 1)
                    }}>Try again</button>
                </div>
            )}
            {roster && (
                <>
                    {!roster.attendance_available && (
                        <p className="haven-rollcall-status haven-rollcall-warning" role="status">
                            <span>Attend check-in status is unavailable.</span>
                            <code>{roster.attendance_error_code ?? 'ATTEND-00'}</code>
                        </p>
                    )}
                    <div className="haven-rollcall-toolbar">
                        <div className="haven-staff-filters haven-rollcall-filters" role="group" aria-label="Roster view">
                            <button type="button" aria-pressed={view === 'attendees'} onClick={() => setView('attendees')}>Attendees</button>
                            <button type="button" aria-pressed={view === 'teams'} onClick={() => setView('teams')}>Teams</button>
                        </div>
                        <input type="search" aria-label="Search roster" placeholder="Search name, team or Attend ID"
                            value={query} onChange={event => setQuery(event.target.value)} />
                    </div>
                    {visible.length === 0 && <p>{search ? 'No matches found.' : 'No attendees on the roster yet.'}</p>}
                    {view === 'attendees' && visible.length > 0 && (
                        <AttendeeList attendees={visible} label="Attendees" attendanceAvailable={roster.attendance_available} />
                    )}
                    {view === 'teams' && [...teams].sort(([a], [b]) => {
                        if (!a) return 1
                        if (!b) return -1
                        return a.localeCompare(b)
                    }).map(([name, members]) => (
                        <div key={name ?? ''} className="haven-rollcall-group">
                            <h3>{name ?? 'No team'}</h3>
                            <AttendeeList attendees={members} label={name ?? 'Attendees without a team'}
                                attendanceAvailable={roster.attendance_available} showTeam={false} />
                        </div>
                    ))}
                </>
            )}
        </section>
    )
}
