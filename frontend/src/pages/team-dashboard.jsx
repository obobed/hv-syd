import { useEffect, useRef, useState } from 'react'
import campfire from '../assets/haven/campfire.png'
import mascot from '../assets/haven/hedgehog.webp'
import './team-dashboard.css'

const inviteCodeLength = 4
const inviteCodePattern = /^[a-zA-Z0-9]{4}$/
const teamLoadError = 'Couldn’t load your team. Try again.'
const inviteError = 'Couldn’t generate an invite code. Try again.'
const disbandError = 'Couldn’t disband your team. Try again.'
const teamRefreshInterval = 15000

function teamRequest(path, signal, { method = 'GET', body } = {}) {
    return fetch(`/api/v1/team/${path}`, {
        method,
        credentials: 'same-origin',
        cache: 'no-store',
        signal: AbortSignal.any([signal, AbortSignal.timeout(10000)]),
        ...(body && { headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) }),
    })
}

async function readTeam(signal, expectTeam = false) {
    const response = await teamRequest('get', signal)
    if (response.status === 404 && !expectTeam) return null
    if (!response.ok) throw new Error(teamLoadError)

    const team = await response.json()
    if (typeof team?.team_name !== 'string' || (team.attendees != null && !Array.isArray(team.attendees))) {
        throw new Error(teamLoadError)
    }
    return team
}

async function readInvite(signal) {
    const response = await teamRequest('otp', signal, { method: 'POST' })
    if (!response.ok) throw new Error(inviteError)

    const code = (await response.text()).trim()
    if (!inviteCodePattern.test(code)) throw new Error(inviteError)
    return code
}

function membershipError(action, status) {
    if (status === 401 || status === 403) return 'Sign in again to manage your team.'
    if (action === 'create' && status === 409) return 'That team name is taken. Try another name.'
    if (action === 'join' && status === 404) return 'That invite code didn’t work. Check it with your teammate.'
    if (action === 'join' && status === 400) return 'Couldn’t join that team. The team may be full.'
    return `Couldn’t ${action} the team. Try again.`
}

function InviteCodeInput({ invalid, disabled }) {
    const [characters, setCharacters] = useState(() => Array(inviteCodeLength).fill(''))
    const inputs = useRef([])

    function enterCharacters(index, value) {
        const code = value.replace(/[^a-zA-Z0-9]/g, '').slice(0, inviteCodeLength)
        if (!code && value) return

        const start = code.length === inviteCodeLength ? 0 : index
        const end = Math.min(start + code.length, inviteCodeLength)
        setCharacters(current => current.map((character, position) => {
            if (position === index && !code) return ''
            if (position >= start && position < end) return code[position - start]
            return character
        }))
        if (code) inputs.current[Math.min(end, inviteCodeLength - 1)].focus()
    }

    function handleKeyDown(event, index) {
        if (event.key === 'Backspace' && !characters[index] && index > 0) {
            event.preventDefault()
            enterCharacters(index - 1, '')
            inputs.current[index - 1].focus()
        }
        if (event.key === 'ArrowLeft' && index > 0) {
            event.preventDefault()
            inputs.current[index - 1].focus()
        }
        if (event.key === 'ArrowRight' && index < inviteCodeLength - 1) {
            event.preventDefault()
            inputs.current[index + 1].focus()
        }
    }

    return (
        <div role="group" aria-labelledby="team-otp-label"
            aria-describedby={invalid ? 'join-team-error' : undefined} className="haven-team-code">
            <input type="hidden" name="otp" value={characters.join('')} />
            {characters.map((character, index) => (
                <input key={index} id={index === 0 ? 'team-otp' : undefined} type="text"
                    aria-label={`Character ${index + 1} of ${inviteCodeLength}`}
                    aria-invalid={invalid || undefined} disabled={disabled}
                    autoComplete={index === 0 ? 'one-time-code' : 'off'}
                    autoCapitalize="none" spellCheck={false} value={character}
                    ref={input => { inputs.current[index] = input }}
                    onFocus={event => event.currentTarget.select()}
                    onChange={event => {
                        const { data, inputType } = event.nativeEvent
                        enterCharacters(index, inputType === 'insertText' && data ? data : event.target.value)
                    }}
                    onKeyDown={event => handleKeyDown(event, index)}
                    onPaste={event => {
                        event.preventDefault()
                        enterCharacters(index, event.clipboardData.getData('text'))
                    }} />
            ))}
        </div>
    )
}

function DisbandTeam({ teamName, pending, disbanding, error, onOpen, onDisband }) {
    const dialog = useRef(null)
    const [confirmation, setConfirmation] = useState('')

    return (
        <>
            <button className="haven-team-disband" type="button" disabled={pending}
                aria-label="Disband team" title="Disband team" onClick={() => {
                    setConfirmation('')
                    onOpen()
                    dialog.current.showModal()
                }}>
                <svg aria-hidden="true" width="22" height="22" viewBox="0 0 24 24" fill="none"
                    stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M4 7h16M9 7V4h6v3M6 7l1 13h10l1-13M10 11v5m4-5v5" />
                </svg>
            </button>
            <dialog ref={dialog} className="haven-team-dialog" aria-labelledby="disband-team-title"
                onCancel={event => { if (disbanding) event.preventDefault() }}>
                <form onSubmit={event => {
                    event.preventDefault()
                    if (confirmation === teamName) onDisband(() => dialog.current?.close())
                }}>
                    <h2 id="disband-team-title">Disband your team?</h2>
                    <p>Everyone will be removed from the team and its invite code will stop working.</p>
                    <label htmlFor="confirm-team-name">Type <strong>{teamName}</strong> to confirm.</label>
                    <input id="confirm-team-name" value={confirmation} disabled={disbanding} autoComplete="off"
                        onChange={event => setConfirmation(event.target.value)} />
                    {error && <p className="haven-team-error" role="alert">{error}</p>}
                    <div className="haven-team-dialog-actions">
                        <button className="haven-team-secondary" type="button" disabled={disbanding}
                            onClick={() => dialog.current.close()}>Cancel</button>
                        <button className="haven-team-primary" type="submit" disabled={pending || confirmation !== teamName}>
                            {disbanding ? 'Disbanding…' : 'Confirm disband'}
                        </button>
                    </div>
                </form>
            </dialog>
        </>
    )
}

function TeamInvite({ disabled }) {
    const [invite, setInvite] = useState(null)
    const [generation, setGeneration] = useState(0)

    useEffect(() => {
        const controller = new AbortController()
        queueMicrotask(async () => {
            // Strict Mode can clean up this effect before the first request starts.
            if (controller.signal.aborted) return
            try {
                const code = await readInvite(controller.signal)
                if (!controller.signal.aborted) setInvite({ code })
            } catch {
                if (!controller.signal.aborted) setInvite({ error: inviteError })
            }
        })
        return () => controller.abort()
    }, [generation])

    return (
        <div className="haven-team-invite">
            <div className="haven-team-code-status" role="status">
                {invite?.code && (
                    <code aria-label={`Invite code ${invite.code.split('').join(' ')}`}>
                        {invite.code.split('').map((character, index) => (
                            <span key={index} aria-hidden="true">{character}</span>
                        ))}
                    </code>
                )}
                {!invite && <p>Generating invite code…</p>}
            </div>
            <button className="haven-team-secondary haven-team-redo" type="button" disabled={disabled || !invite}
                aria-label="Generate new code" title="Generate new code"
                onClick={() => {
                    setInvite(null)
                    setGeneration(current => current + 1)
                }}>
                <svg aria-hidden="true" width="22" height="22" viewBox="0 0 24 24" fill="none"
                    stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M20 4v6h-6M20 10a8 8 0 1 0-1.3 8.1" />
                </svg>
            </button>
            {invite?.error && <p className="haven-team-error" role="alert">{invite.error}</p>}
        </div>
    )
}

export default function TeamDashboard() {
    const [team, setTeam] = useState(undefined)
    const [pending, setPending] = useState('')
    const [error, setError] = useState(null)
    const actionRequest = useRef(null)
    const completedMembershipAction = useRef('')
    const currentTeamName = team?.team_name

    useEffect(() => {
        const controller = new AbortController()
        readTeam(controller.signal).then(result => {
            if (!controller.signal.aborted) setTeam(result)
        }).catch(() => {
            if (!controller.signal.aborted) setError({ action: 'load', message: teamLoadError })
        })

        return () => {
            controller.abort()
            actionRequest.current?.abort()
        }
    }, [])

    useEffect(() => {
        if (!currentTeamName || pending) return

        let request
        function updateRoster() {
            if (document.hidden || actionRequest.current || request) return
            const controller = new AbortController()
            request = controller
            readTeam(controller.signal).then(result => {
                if (controller.signal.aborted || actionRequest.current) return
                setTeam(result)
            }).catch(() => {
                // Keep the last roster visible when an automatic check fails.
            }).finally(() => { request = null })
        }

        const interval = window.setInterval(updateRoster, teamRefreshInterval)
        window.addEventListener('focus', updateRoster)
        document.addEventListener('visibilitychange', updateRoster)
        return () => {
            request?.abort()
            window.clearInterval(interval)
            window.removeEventListener('focus', updateRoster)
            document.removeEventListener('visibilitychange', updateRoster)
        }
    }, [currentTeamName, pending])

    async function runAction(action, task, fallback) {
        if (actionRequest.current) return

        const controller = new AbortController()
        actionRequest.current = controller
        setPending(action)
        setError(null)

        try {
            await task(controller.signal)
        } catch {
            if (!controller.signal.aborted) setError({ action, message: fallback })
        } finally {
            actionRequest.current = null
            if (!controller.signal.aborted) setPending('')
        }
    }

    async function refreshTeam(signal) {
        const membershipAction = completedMembershipAction.current
        const result = await readTeam(signal, Boolean(membershipAction))
        if (signal.aborted) return
        completedMembershipAction.current = ''
        setTeam(result)
    }

    function disbandTeam(closeDialog) {
        if (team?.can_disband !== true) return
        void runAction('disband', async signal => {
            const response = await teamRequest('disband', signal, {
                method: 'POST',
                body: { teamName: team.team_name },
            })
            if (!response.ok || response.headers.get('content-type')?.includes('text/html')) {
                throw new Error(disbandError)
            }
            if (signal.aborted) return
            closeDialog()
            setTeam(null)
        }, disbandError)
    }

    function submitMembership(event, action) {
        event.preventDefault()
        if (actionRequest.current) return
        const field = action === 'create' ? 'teamName' : 'otp'
        const value = new FormData(event.currentTarget).get(field).trim()
        if (action === 'create' ? !value : !inviteCodePattern.test(value)) {
            setError({ action, message: action === 'create' ? 'Enter a team name.' : 'Enter the four-character invite code.' })
            return
        }

        void runAction(action, async signal => {
            const response = await teamRequest(action, signal, { method: 'POST', body: { [field]: value } })
            if (signal.aborted) return
            if (response.status === 409) {
                let existingTeam
                try {
                    existingTeam = await readTeam(signal)
                } catch {
                    if (!signal.aborted) setError({ action, message: teamLoadError })
                    return
                }
                if (signal.aborted) return
                if (existingTeam) {
                    setTeam(existingTeam)
                } else {
                    setError({ action, message: membershipError(action, response.status) })
                }
                return
            }
            if (!response.ok) {
                setError({ action, message: membershipError(action, response.status) })
                return
            }

            completedMembershipAction.current = action
            setTeam(undefined)
            try {
                await refreshTeam(signal)
            } catch {
                if (!signal.aborted) setError({ action: 'load', message: teamLoadError })
            }
        }, `Couldn’t ${action} the team. Try again.`)
    }

    const members = team?.attendees ?? []
    const loading = team === undefined && !error

    return (
        <main className="haven-team" aria-label="Team dashboard">
            <div className="haven-team-shell">
                {team === undefined && (
                    <section className="haven-team-card haven-team-loading" aria-label="Team status">
                        {loading ? <p role="status">Loading your team…</p> : (
                            <>
                                <p role="alert">{error.message}</p>
                                <button className="haven-team-secondary" type="button" disabled={Boolean(pending)}
                                    onClick={() => void runAction('load', refreshTeam, teamLoadError)}>Try again</button>
                            </>
                        )}
                    </section>
                )}

                {team === null && (
                    <div className="haven-team-setup">
                        <form className="haven-team-card" aria-labelledby="join-team-title" onSubmit={event => submitMembership(event, 'join')}>
                            <div className="haven-team-avatars" aria-hidden="true">
                                <span><img src={mascot} alt="" /></span>
                                <span><img src={mascot} alt="" /></span>
                                <span><img src={mascot} alt="" /></span>
                                <span className="haven-team-avatar-empty">
                                    <svg width="16" height="16" viewBox="0 0 16 16" fill="none"
                                        stroke="currentColor" strokeWidth="2" strokeLinecap="round">
                                        <path d="M8 3v10M3 8h10" />
                                    </svg>
                                </span>
                            </div>
                            <h2 id="join-team-title">Join a team</h2>
                            <p>Enter the four-character code from your friend.</p>
                            <label id="team-otp-label" htmlFor="team-otp">Invite code</label>
                            <InviteCodeInput invalid={error?.action === 'join'} disabled={Boolean(pending)} />
                            <button className="haven-team-primary" type="submit" disabled={Boolean(pending)}>
                                {pending === 'join' ? 'Joining…' : 'Join team'}
                            </button>
                            {error?.action === 'join' && <p className="haven-team-error" role="alert" id="join-team-error">{error.message}</p>}
                        </form>

                        <span className="haven-team-or">or</span>

                        <form className="haven-team-card" aria-labelledby="create-team-title" onSubmit={event => submitMembership(event, 'create')}>
                            <div className="haven-team-campfire">
                                <img src={campfire} alt="" width="120" height="174" />
                            </div>
                            <h2 id="create-team-title">Create a team</h2>
                            <p>Pick a team name to get an invite code to share.</p>
                            <label htmlFor="team-name">Team name</label>
                            <input id="team-name" name="teamName" type="text" placeholder="e.g. Gumleaf Coders" required
                                autoComplete="off" aria-invalid={error?.action === 'create' || undefined}
                                aria-describedby={error?.action === 'create' ? 'create-team-error' : undefined} />
                            <button className="haven-team-primary" type="submit" disabled={Boolean(pending)}>
                                {pending === 'create' ? 'Creating…' : 'Create team'}
                            </button>
                            {error?.action === 'create' && <p className="haven-team-error" role="alert" id="create-team-error">{error.message}</p>}
                        </form>
                    </div>
                )}

                {team && (
                    <section className="haven-team-card haven-team-roster" aria-labelledby="team-name-title">
                        <div className="haven-team-name-row">
                            <h2 id="team-name-title">{team.team_name}</h2>
                            {team.can_disband === true && (
                                <DisbandTeam key={team.team_name} teamName={team.team_name} pending={Boolean(pending)}
                                    disbanding={pending === 'disband'}
                                    error={error?.action === 'disband' ? error.message : undefined}
                                    onOpen={() => setError(current => current?.action === 'disband' ? null : current)}
                                    onDisband={disbandTeam} />
                            )}
                        </div>
                        <h3>Team members</h3>
                        {members.length ? (
                            <ul className="haven-team-members" role="list">
                                {members.map(member => (
                                    <li key={member.id}>
                                        <span className="haven-team-avatar"><img src={mascot} alt="" /></span>
                                        {[member.first_name, member.last_name].filter(Boolean).join(' ')}
                                    </li>
                                ))}
                            </ul>
                        ) : <p>No members listed yet.</p>}
                        <TeamInvite key={team.team_name} disabled={Boolean(pending)} />
                    </section>
                )}
            </div>
        </main>
    )
}
