import Cookies from 'js-cookie'

const SESSION_KEY = 'haven-display-session'

export function readDisplaySession() {
    try {
        return JSON.parse(sessionStorage.getItem(SESSION_KEY))
    } catch {
        return null
    }
}

export async function saveDisplaySession({ staff, payload, signal }) {
    // The team endpoint includes first names for attendees who have a team.
    // Store only the display name for the navbar.
    let name = staff ? payload.username?.trim() || 'Staff' : 'Attendee'

    if (!staff) {
        try {
            const response = await fetch('/api/v1/team/get', {
                credentials: 'include',
                signal: AbortSignal.any([signal, AbortSignal.timeout(1500)]),
            })
            if (response.ok) {
                const team = await response.json()
                const attendee = team.attendees?.find(member => member.id === payload.id)
                name = attendee?.first_name?.trim() || name
            }
        } catch {
            // A missing or unavailable team does not prevent sign-in.
        }
    }

    if (!signal.aborted) {
        try {
            sessionStorage.setItem(SESSION_KEY, JSON.stringify({ name }))
        } catch {
            // The session cookie still works if browser storage is unavailable.
        }
    }
}

export function clearSession() {
    // Login cookies without a Path default to the directory of the login endpoint.
    Cookies.remove('attendee-id', { path: '/api/v1' })
    Cookies.remove('admin-token', { path: '/api/admin' })
    Cookies.remove('attendee-id')
    Cookies.remove('admin-token')
    try {
        sessionStorage.removeItem(SESSION_KEY)
    } catch {
        // There is no display state to clear when storage is unavailable.
    }
}
