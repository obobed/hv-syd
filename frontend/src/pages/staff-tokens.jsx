import { useEffect, useRef, useState } from 'react'

const secondsPerUnit = { minutes: 60, hours: 3600, days: 86400 }
const expiryFormat = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' })

const tokenIconPaths = {
    add: 'M12 5v14M5 12h14',
    person: 'M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2M16 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0Z',
    copy: 'M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2m2-8h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2h-8a2 2 0 0 1-2-2v-8a2 2 0 0 1 2-2Z',
    check: 'm5 12 4 4L19 6',
    pencil: 'm14 6 4 4M4 20l1-5L16 4a2.8 2.8 0 0 1 4 4L9 19l-5 1Z',
}

function TokenIcon({ name }) {
    return (
        <svg className="haven-staff-token-icon" aria-hidden="true" viewBox="0 0 24 24" width="18" height="18"
            fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            {name === 'key'
                ? <><circle cx="16" cy="8" r="5" /><path d="m12.5 11.5-8 8V16l3.5-3.5H11" /></>
                : <path d={tokenIconPaths[name]} />}
        </svg>
    )
}

async function tokenRequest(path, signal, body) {
    const response = await fetch(`/api/admin/token/${path}`, {
        credentials: 'same-origin', cache: 'no-store',
        signal: AbortSignal.any([signal, AbortSignal.timeout(10000)]),
        ...(body && {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body),
        }),
    })
    if (!response.ok || response.headers.get('content-type')?.includes('text/html')) {
        throw new Error('Token request failed')
    }
    return response
}

async function readTokens(signal) {
    const response = await tokenRequest('list', signal)
    const records = await response.json()
    if (!records || typeof records !== 'object' || Array.isArray(records)) throw new Error('Expected a token list')
    return records
}

export default function StaffTokens() {
    const [tokens, setTokens] = useState(null)
    const [loadedAt, setLoadedAt] = useState(0)
    const [loadError, setLoadError] = useState(false)
    const [loadAttempt, setLoadAttempt] = useState(0)
    const [target, setTarget] = useState(null)
    const [level, setLevel] = useState('admin')
    const [busy, setBusy] = useState(false)
    const [error, setError] = useState('')
    const [notice, setNotice] = useState('')
    const [copyStatus, setCopyStatus] = useState('')
    const dialog = useRef(null)
    const mutation = useRef(null)

    useEffect(() => {
        const controller = new AbortController()
        const { signal } = controller
        readTokens(signal).then(records => {
            if (!signal.aborted) {
                setTokens(records)
                setLoadedAt(Date.now())
            }
        }).catch(() => {
            if (!signal.aborted) setLoadError(true)
        })
        return () => controller.abort()
    }, [loadAttempt])

    useEffect(() => () => mutation.current?.abort(), [])

    useEffect(() => {
        if (target && !dialog.current.open) dialog.current.showModal()
    }, [target])

    function openDialog(token) {
        setTarget({ token })
        setLevel('admin')
        setError('')
        setCopyStatus('')
    }

    async function save(event) {
        event.preventDefault()
        if (mutation.current) return
        const fields = new FormData(event.currentTarget)
        const seconds = Number(fields.get('duration')) * secondsPerUnit[fields.get('unit')]
        if (!Number.isSafeInteger(seconds) || seconds <= 0) {
            setError('Enter a valid duration.')
            return
        }

        const { token } = target
        const controller = new AbortController()
        const { signal } = controller
        mutation.current = controller
        setBusy(true)
        setError('')
        try {
            await tokenRequest(token ? encodeURIComponent(token) : 'create', signal, token
                ? { add: [seconds, 0] }
                : { level, valid_for: [seconds, 0] })
            if (signal.aborted) return
            dialog.current.close()
            setNotice(token ? 'Token extended.' : level === 'admin' ? 'Staff token created.' : 'Attendee token created.')
            try {
                const records = await readTokens(signal)
                if (!signal.aborted) {
                    setTokens(records)
                    setLoadedAt(Date.now())
                }
            } catch {
                if (!signal.aborted) setLoadError(true)
            }
        } catch {
            if (!signal.aborted) setError('Couldn’t save the token. Check the token list before trying again.')
        } finally {
            mutation.current = null
            if (!signal.aborted) setBusy(false)
        }
    }

    async function copy(token) {
        try {
            await navigator.clipboard.writeText(token)
            setCopyStatus('Token copied.')
        } catch {
            setCopyStatus('Couldn’t copy the token. Select the token ID to copy it.')
        }
    }

    const entries = Object.entries(tokens ?? {}).sort(([, a], [, b]) => Date.parse(a.exp) - Date.parse(b.exp))
    const action = target?.token ? 'Extend token' : 'New token'

    return (
        <section className="haven-team-card haven-staff-tokens" aria-labelledby="staff-tokens-title">
            <header className="haven-staff-heading">
                <h2 id="staff-tokens-title">Access tokens</h2>
                <button type="button" className="haven-team-primary haven-staff-icon-label" disabled={!tokens || loadError || busy}
                    onClick={() => openDialog(null)}><TokenIcon name="add" />New token</button>
            </header>
            <p className="haven-staff-notice" role="status">{notice}</p>
            {!tokens && !loadError && <p role="status">Loading tokens…</p>}
            {loadError && (
                <div className="haven-staff-actions">
                    <p className="haven-team-error" role="alert">Couldn’t load the token list.</p>
                    <button type="button" className="haven-team-secondary" onClick={() => {
                        setLoadError(false)
                        setLoadAttempt(current => current + 1)
                    }}>Try again</button>
                </div>
            )}
            {tokens && !loadError && entries.length === 0 && <p>No access tokens.</p>}
            {entries.length > 0 && (
                <ul className="haven-staff-token-list" role="list">
                    {entries.map(([token, metadata], index) => {
                        const expiry = new Date(metadata.exp)
                        const isStaff = metadata.level === 'admin'
                        const type = isStaff ? 'Staff token' : 'Attendee token'
                        return (
                            <li key={token} className="haven-staff-token">
                                <TokenIcon name={isStaff ? 'key' : 'person'} />
                                <div className="haven-staff-token-details">
                                    <strong>{type}</strong>
                                    <p>{expiry.getTime() <= loadedAt ? 'Expired' : 'Expires'} <time dateTime={metadata.exp}>{expiryFormat.format(expiry)}</time></p>
                                </div>
                                <button type="button" className="haven-team-secondary haven-staff-icon-button"
                                    aria-label={`View and edit ${type.toLowerCase()} ${index + 1}`} title="View and edit token"
                                    disabled={busy || loadError} onClick={() => openDialog(token)}>
                                    <TokenIcon name="pencil" />
                                </button>
                            </li>
                        )
                    })}
                </ul>
            )}
            <dialog ref={dialog} className="haven-team-dialog" aria-labelledby="staff-token-action"
                onCancel={event => { if (busy) event.preventDefault() }} onClose={() => setTarget(null)}>
                {target && (
                    <form onSubmit={save}>
                        <h3 id="staff-token-action">{action}</h3>
                        {target.token && (
                            <>
                                <div className="haven-staff-token-secret">
                                    <code>{target.token}</code>
                                    <button type="button" className="haven-team-secondary haven-staff-icon-button"
                                        aria-label="Copy token" title="Copy token" onClick={() => void copy(target.token)}>
                                        <TokenIcon name={copyStatus === 'Token copied.' ? 'check' : 'copy'} />
                                    </button>
                                </div>
                                <p className="haven-staff-copy-status" role="status">{copyStatus}</p>
                            </>
                        )}
                        {!target.token && (
                            <>
                                <label htmlFor="staff-token-level">Token type</label>
                                <select id="staff-token-level" value={level} disabled={busy}
                                    onChange={event => setLevel(event.target.value)}>
                                    <option value="admin">Staff</option>
                                    <option value="attendee">Attendee</option>
                                </select>
                                {level === 'attendee' && <p>Attendee sign-in currently uses an Attend ID.</p>}
                            </>
                        )}
                        <label htmlFor="staff-token-duration">{target.token ? 'Add time' : 'Valid for'}</label>
                        <div className="haven-staff-token-duration">
                            <input id="staff-token-duration" name="duration" type="number" min="1" step="1"
                                defaultValue="1" required disabled={busy} />
                            <select name="unit" aria-label="Duration unit" defaultValue="hours" disabled={busy}>
                                <option value="minutes">Minutes</option>
                                <option value="hours">Hours</option>
                                <option value="days">Days</option>
                            </select>
                        </div>
                        {error && <p role="alert" className="haven-team-error">{error}</p>}
                        <div className="haven-team-dialog-actions">
                            <button type="button" className="haven-team-secondary" disabled={busy}
                                onClick={() => dialog.current.close()}>Cancel</button>
                            <button type="submit" className="haven-team-primary" disabled={busy}>
                                {busy ? 'Saving…' : target.token ? action : 'Generate token'}
                            </button>
                        </div>
                    </form>
                )}
            </dialog>
        </section>
    )
}
