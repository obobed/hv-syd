import Cookies from 'js-cookie'

const COOKIE_PATHS = { 'attendee-id': '/api/v1', 'admin-token': '/api/admin' }

function removeCookie(name) {
    Cookies.remove(name, { path: COOKIE_PATHS[name] })
    Cookies.remove(name)
}

export function clearPreviousRole(staff) {
    removeCookie(staff ? 'attendee-id' : 'admin-token')
}

async function fetchSession(path, signal) {
    const response = await fetch(path, { credentials: 'same-origin', cache: 'no-store', signal })
    if (!response.ok || !response.headers.get('content-type')?.includes('application/json')) return null
    const session = await response.json()
    return typeof session.name === 'string' ? session : null
}

export async function readSession(signal) {
    const lookupSignal = AbortSignal.any([signal, AbortSignal.timeout(5000)])
    const [staff, attendee] = await Promise.allSettled([
        fetchSession('/api/admin/session', lookupSignal),
        fetchSession('/api/v1/session', lookupSignal),
    ])
    if (staff.status === 'fulfilled' && staff.value) return { name: 'Staff' }
    if (attendee.status === 'fulfilled' && attendee.value) {
        return { name: attendee.value.name.trim() || 'Attendee' }
    }
    return null
}

export function clearSession() {
    // Cookies without an explicit Path use the directory of the login endpoint.
    removeCookie('attendee-id')
    removeCookie('admin-token')
}
