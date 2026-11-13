import { useEffect, useRef, useState } from 'react'
import mascot from '../assets/haven/hedgehog.webp'

const queueError = 'Couldn’t load the review queue.'
const filters = [
    ['unreviewed', 'Unreviewed'],
    ['approved', 'Approved'],
    ['rejected', 'Rejected'],
]

const reviewIconPaths = {
    approved: 'm5 12 4 4L19 6',
    rejected: 'm6 6 12 12M18 6 6 18',
    view: 'M2 12s4-6 10-6 10 6 10 6-4 6-10 6S2 12 2 12Zm13 0a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z',
    collapse: 'm6 15 6-6 6 6',
    download: 'M12 3v12m-4-4 4 4 4-4M4 16v4h16v-4',
    external: 'M13 5h6v6m0-6-8 8m5 0v6H5V8h6',
    milestone: 'M5 21V4m0 1c5-4 9 4 14 0v9c-5 4-9-4-14 0',
}

function ReviewIcon({ name }) {
    return (
        <svg aria-hidden="true" width="16" height="16" viewBox="0 0 24 24"
            fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <path d={reviewIconPaths[name]} />
        </svg>
    )
}

function attachmentDetails(attachment) {
    if (!attachment) return null
    const privateAsset = attachment.startsWith('/api/admin/asset/')
    try {
        const url = privateAsset
            ? new URL(attachment.split('/').map(encodeURIComponent).join('/'), window.location.origin)
            : new URL(attachment)
        if (privateAsset && url.origin === window.location.origin && url.pathname.startsWith('/api/admin/asset/')) {
            const path = url.pathname.toLowerCase()
            if (/\.(avif|gif|heic|heif|jpe?g|png|webp)$/.test(path)) return { url: url.href, kind: 'image' }
            if (/\.(m4v|mov|mp4|ogv|webm)$/.test(path)) return { url: url.href, kind: 'video' }
            return { url: url.href, kind: 'file' }
        }
        if (!privateAsset && (url.protocol === 'https:' || url.protocol === 'http:')) {
            return { url: url.href, kind: 'link' }
        }
    } catch {
        return null
    }
    return null
}

function SubmissionAttachment({ attachment, teamName, children }) {
    const media = attachmentDetails(attachment)
    const [failed, setFailed] = useState(false)
    return (
        <div className="haven-staff-attachment">
            {!media && <p>{attachment ? 'This attachment is unavailable.' : 'No file attached.'}</p>}
            {media?.kind === 'image' && !failed && (
                <img src={media.url} alt={`Submission from ${teamName}`} onError={() => setFailed(true)} />
            )}
            {media?.kind === 'video' && !failed && (
                <video src={media.url} controls playsInline preload="metadata"
                    aria-label={`Submission from ${teamName}`} onError={() => setFailed(true)} />
            )}
            {failed && <p>Preview unavailable.</p>}
            <div className="haven-staff-actions haven-staff-review-actions">
                {media?.kind === 'link' && (
                    <a className="haven-team-secondary haven-staff-icon-label" href={media.url}
                        target="_blank" rel="noopener noreferrer"><ReviewIcon name="external" />Open link</a>
                )}
                {media && media.kind !== 'link' && (
                    <a className="haven-team-secondary haven-staff-icon-label" href={media.url} download>
                        <ReviewIcon name="download" />Download
                    </a>
                )}
                {children}
            </div>
        </div>
    )
}

function TeamAvatars({ members }) {
    if (!members.length) return null

    return (
        <span className="haven-team-avatars haven-staff-avatars" role="img"
            aria-label={`${members.length} team ${members.length === 1 ? 'member' : 'members'}`}>
            {members.map(member => (
                <span key={member.id} title={[member.first_name, member.last_name].filter(Boolean).join(' ')}>
                    <img src={mascot} alt="" />
                </span>
            ))}
        </span>
    )
}

export default function StaffSubmissions() {
    const [queue, setQueue] = useState(null)
    const [filter, setFilter] = useState('unreviewed')
    const [loadError, setLoadError] = useState('')
    const [rosters, setRosters] = useState(null)
    const [rosterError, setRosterError] = useState(false)
    const [reviewing, setReviewing] = useState(null)
    const [approving, setApproving] = useState(false)
    const [approvalError, setApprovalError] = useState('')
    const [notice, setNotice] = useState('')
    const approval = useRef(null)
    const heading = useRef(null)

    useEffect(() => {
        const controller = new AbortController()
        const { signal } = controller
        async function readList(path) {
            const response = await fetch(`/api/admin/${path}/list`, {
                credentials: 'same-origin', cache: 'no-store',
                signal: AbortSignal.any([signal, AbortSignal.timeout(10000)]),
            })
            if (!response.ok) throw new Error('List request failed')
            const records = await response.json()
            if (!Array.isArray(records)) throw new Error('Expected a list')
            return records
        }

        readList('submission').then(submissions => {
            if (!signal.aborted) setQueue(submissions)
        }).catch(() => {
            if (!signal.aborted) setLoadError(queueError)
        })
        readList('attendee').then(attendees => {
            if (signal.aborted) return
            const membersByTeam = new Map()
            for (const member of attendees) {
                if (member.team_name == null) continue
                const members = membersByTeam.get(member.team_name) ?? []
                members.push(member)
                membersByTeam.set(member.team_name, members)
            }
            setRosters(membersByTeam)
        }).catch(() => {
            if (!signal.aborted) setRosterError(true)
        })
        return () => controller.abort()
    }, [])

    useEffect(() => () => approval.current?.abort(), [])

    async function decide(submission, status) {
        if (approval.current) return
        const controller = new AbortController()
        approval.current = controller
        setApproving(true)
        setApprovalError('')
        setNotice('')
        try {
            const response = await fetch(`/api/admin/submission/${encodeURIComponent(submission.id)}`, {
                method: 'POST',
                credentials: 'same-origin',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(status === 'approved' ? { approved: true } : { status }),
                signal: AbortSignal.any([controller.signal, AbortSignal.timeout(10000)]),
            })
            if (!response.ok || response.headers.get('content-type')?.includes('text/html')) {
                throw new Error('Review update failed')
            }
            if (controller.signal.aborted) return
            setQueue(current => current.map(item => item.id === submission.id
                ? { ...item, approved: status === 'approved', rejected: status === 'rejected' }
                : item))
            setReviewing(null)
            setNotice(`${status === 'approved' ? 'Approved' : 'Rejected'} submission from ${submission.team_name}.`)
            heading.current?.focus()
        } catch {
            if (!controller.signal.aborted) setApprovalError('Couldn’t save the decision. Try again or reload the page.')
        } finally {
            approval.current = null
            if (!controller.signal.aborted) setApproving(false)
        }
    }

    const byStatus = { unreviewed: [], approved: [], rejected: [] }
    for (const submission of queue ?? []) {
        const status = submission.approved ? 'approved' : submission.rejected ? 'rejected' : 'unreviewed'
        byStatus[status].push(submission)
    }
    const visible = queue ? byStatus[filter] : null

    return (
        <section className="haven-team-card haven-staff-queue" aria-labelledby="staff-queue-title">
            <h2 id="staff-queue-title" ref={heading} tabIndex={-1}>Review submissions</h2>
            <div className="haven-staff-filters" role="group" aria-label="Filter submissions">
                {filters.map(([value, label]) => (
                    <button key={value} type="button" aria-pressed={filter === value} disabled={approving}
                        onClick={() => {
                            setFilter(value)
                            setReviewing(null)
                            setApprovalError('')
                            setNotice('')
                        }}>
                        {label}
                        {queue && <span className="haven-staff-filter-count">{byStatus[value].length}</span>}
                    </button>
                ))}
            </div>
            <p className="haven-staff-notice" role="status">{notice}</p>
            {!queue && !loadError && <p role="status">Loading submissions…</p>}
            {loadError && <p className="haven-team-error" role="alert">{loadError}</p>}
            {visible?.length === 0 && <p>{filter === 'unreviewed' ? 'All caught up.' : `No ${filter} submissions.`}</p>}
            {visible?.length > 0 && (
                <>
                    {rosterError && <p role="status">Team rosters couldn’t load.</p>}
                    <ul className="haven-staff-review-list" role="list">
                        {visible.map(submission => {
                            const expanded = reviewing === submission.id
                            const panelId = `staff-review-${submission.id}`
                            return (
                                <li key={submission.id} className="haven-staff-review">
                                    <div className="haven-staff-heading">
                                        <div>
                                            <div className="haven-staff-team-name">
                                                <h3>{submission.team_name}</h3>
                                                {rosters && <TeamAvatars members={rosters.get(submission.team_name) ?? []} />}
                                            </div>
                                            <p className="haven-staff-icon-label"><ReviewIcon name="milestone" />Milestone {submission.milestone}</p>
                                        </div>
                                        <button type="button" className="haven-team-secondary haven-staff-icon-label"
                                            aria-expanded={expanded} aria-controls={panelId}
                                            aria-label={`${expanded ? 'Close review for' : filter === 'unreviewed' ? 'Review' : 'View'} ${submission.team_name}, milestone ${submission.milestone}`}
                                            disabled={approving} onClick={() => {
                                                setApprovalError('')
                                                setReviewing(expanded ? null : submission.id)
                                            }}>
                                            <ReviewIcon name={expanded ? 'collapse' : 'view'} />
                                            {expanded ? 'Close' : filter === 'unreviewed' ? 'Review' : 'View'}
                                        </button>
                                    </div>
                                    <div id={panelId} hidden={!expanded}>
                                        {expanded && (
                                            <div className="haven-staff-review-details">
                                                {submission.description && <p className="haven-staff-description">{submission.description}</p>}
                                                <SubmissionAttachment key={submission.attachment}
                                                    attachment={submission.attachment} teamName={submission.team_name}>
                                                    {filter === 'unreviewed' && (
                                                        <>
                                                            <button type="button" className="haven-team-secondary haven-staff-icon-label"
                                                                disabled={approving} onClick={() => void decide(submission, 'rejected')}>
                                                                <ReviewIcon name="rejected" />Reject
                                                            </button>
                                                            <button type="button" className="haven-team-primary haven-staff-icon-label"
                                                                disabled={approving} onClick={() => void decide(submission, 'approved')}>
                                                                <ReviewIcon name="approved" />Approve
                                                            </button>
                                                        </>
                                                    )}
                                                </SubmissionAttachment>
                                                {approvalError && <p className="haven-team-error" role="alert">{approvalError}</p>}
                                            </div>
                                        )}
                                    </div>
                                </li>
                            )
                        })}
                    </ul>
                </>
            )}
        </section>
    )
}
