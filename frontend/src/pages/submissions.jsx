import { useEffect, useRef, useState } from 'react'
import './submissions.css'

const loadError = 'Couldn’t load milestones and submissions.'
const createError = 'Couldn’t submit your work. Check your submissions before trying again.'

async function request(path, signal, body) {
    const response = await fetch(`/api/v1/${path}`, {
        credentials: 'same-origin',
        cache: 'no-store',
        signal: AbortSignal.any([signal, AbortSignal.timeout(path === 'upload' ? 120000 : 10000)]),
        ...(body && {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body),
        }),
    })
    if (response.status === 401 || response.status === 403) {
        throw new Error('Sign in again to view or submit your work.')
    }
    return response
}

async function readList(path, signal) {
    const response = await request(path, signal)
    if (!response.ok || !response.headers.get('content-type')?.includes('application/json')) {
        throw new Error(loadError)
    }
    const records = await response.json()
    if (!Array.isArray(records)) throw new Error(loadError)
    return records
}

async function readResponseText(response) {
    return response.headers.get('content-type')?.includes('application/json')
        ? response.json()
        : (await response.text()).trim()
}

function SubmissionAction({ milestone, submission, onSubmitted, headingRef }) {
    const [file, setFile] = useState(null)
    const [error, setError] = useState('')
    const [pending, setPending] = useState(false)
    const requestRef = useRef(null)
    const fileInput = useRef(null)
    const pickerButton = useRef(null)
    const uploadedFile = useRef(null)

    useEffect(() => () => requestRef.current?.abort(), [])

    function selectFile(event) {
        const selected = event.currentTarget.files[0]
        event.currentTarget.value = ''
        if (!selected) return
        if (!/^(image|video)\//.test(selected.type) || !selected.size) {
            setError('Choose an image or video file that isn’t empty.')
            return
        }
        uploadedFile.current = null
        setError('')
        setFile(selected)
    }

    async function submit(event) {
        event.preventDefault()
        if (!file || requestRef.current) return
        setPending(true)
        setError('')
        const controller = new AbortController()
        requestRef.current = controller
        try {
            if (!uploadedFile.current) {
                const content = Array.from(new Uint8Array(await file.arrayBuffer()))
                const upload = await request('upload', controller.signal, {
                    filename: `${crypto.randomUUID()}-${file.name}`,
                    content,
                })
                if (!upload.ok) throw new Error(createError)
                const attachment = await readResponseText(upload)
                if (typeof attachment !== 'string' || !attachment.startsWith('/api/admin/asset/')) {
                    throw new Error(createError)
                }
                uploadedFile.current = attachment
            }
            const response = await request('submission/create', controller.signal, {
                milestone: milestone.id,
                description: null,
                attachment: uploadedFile.current,
            })
            if (!response.ok) throw new Error(createError)
            const id = await readResponseText(response)
            if (typeof id !== 'string' || !/^[\da-f]{8}(-[\da-f]{4}){3}-[\da-f]{12}$/i.test(id)) {
                throw new Error(createError)
            }
            if (!controller.signal.aborted) {
                onSubmitted({ id, milestone: milestone.id, attachment: uploadedFile.current, approved: false })
                setFile(null)
                headingRef.current?.focus({ preventScroll: true })
            }
        } catch {
            if (!controller.signal.aborted) setError(createError)
        } finally {
            requestRef.current = null
            if (!controller.signal.aborted) setPending(false)
        }
    }

    if (submission) return <MilestoneStatus submission={submission} />

    return (
        <>
            <button type="button" className="haven-milestone-submit"
                ref={pickerButton} disabled={pending}
                aria-label={file ? `Change file for ${milestone.name}` : `Submit ${milestone.name}`}
                onClick={() => fileInput.current.click()}>
                <span>{file ? 'Change file' : 'Submit'}</span>
            </button>
            <input ref={fileInput} type="file" accept="image/*,video/*" hidden disabled={pending} onChange={selectFile} />
            {(file || error) && (
                <form className="haven-submission-form" onSubmit={submit}>
                    {file && (
                        <>
                            <p role="status">{file.name}</p>
                            <div className="haven-submission-actions">
                                <button type="submit" className="haven-team-primary" disabled={pending}>
                                    {pending ? 'Submitting…' : 'Submit for review'}
                                </button>
                                <button type="button" className="haven-team-secondary" disabled={pending} onClick={() => {
                                    setFile(null)
                                    setError('')
                                    pickerButton.current?.focus()
                                }}>Cancel</button>
                            </div>
                        </>
                    )}
                    {error && <p role="alert" className="haven-team-error">{error}</p>}
                </form>
            )}
        </>
    )
}

function milestoneStatus(submission) {
    return submission?.approved ? 'approved' : submission ? 'review' : 'ready'
}

function MilestoneStatus({ submission }) {
    const status = milestoneStatus(submission)
    const label = status === 'approved' ? 'Approved' : 'In review'
    return <span className="haven-milestone-status" data-status={status}>{label}</span>
}

function MilestoneDetails({ milestone, submission }) {
    return (
        <div className="haven-milestone-content">
            {milestone.description && <p>{milestone.description}</p>}
            {milestone.optional && <p>Optional milestone</p>}
            {milestone.requirements?.length > 0 && (
                <ul aria-label="Requirements">
                    {milestone.requirements.map((requirement, index) => <li key={index}>{requirement}</li>)}
                </ul>
            )}
            {milestone.prizes?.length > 0 && <p><strong>Prizes</strong> · {milestone.prizes.join(', ')}</p>}
            {submission?.description && <p className="haven-submission-description">{submission.description}</p>}
            {submission?.attachment && /^https?:\/\//i.test(submission.attachment) && (
                <a className="haven-submission-link" href={submission.attachment}
                    target="_blank" rel="noopener noreferrer">View submitted work</a>
            )}
            {submission?.attachment?.startsWith('/api/admin/asset/') && <p>File attached</p>}
        </div>
    )
}

function MilestoneRow({ entry, number, onSubmitted }) {
    const { milestone, submission } = entry
    const heading = useRef(null)
    const hasDetails = milestone.description || milestone.optional || milestone.requirements?.length
        || milestone.prizes?.length || submission?.description || submission?.attachment

    return (
        <li className="haven-milestone-row">
            <span className="haven-milestone-number" aria-hidden="true">{number}</span>
            {hasDetails ? (
                <details>
                    <summary className="haven-milestone-summary" ref={heading}>{milestone.name}</summary>
                    <MilestoneDetails milestone={milestone} submission={submission} />
                </details>
            ) : <span className="haven-milestone-title" tabIndex={-1} ref={heading}>{milestone.name}</span>}
            <SubmissionAction milestone={milestone} submission={submission}
                onSubmitted={onSubmitted} headingRef={heading} />
        </li>
    )
}

export default function Submissions({ teamName }) {
    const [data, setData] = useState(null)
    const [error, setError] = useState('')
    const [attempt, setAttempt] = useState(0)
    const [notice, setNotice] = useState('')
    const showcaseHeading = useRef(null)

    useEffect(() => {
        let controller
        async function load() {
            controller?.abort()
            const requestController = new AbortController()
            controller = requestController
            const { signal } = requestController
            try {
                const [milestones, submissions] = await Promise.all([
                    readList('milestones', signal),
                    readList('submission/list', signal),
                ])
                if (!signal.aborted) {
                    const teamSubmissions = submissions.filter(submission => submission.team_name === teamName)
                    setData({ milestones, submissions: teamSubmissions })
                    setError('')
                }
            } catch (error) {
                if (!signal.aborted) {
                    setError(error instanceof TypeError || error.name === 'TimeoutError' ? loadError : error.message)
                }
            }
        }
        void load()
        window.addEventListener('focus', load)
        return () => {
            controller?.abort()
            window.removeEventListener('focus', load)
        }
    }, [attempt, teamName])

    function submissionCreated(submission) {
        setData(current => ({
            ...current,
            submissions: [...current.submissions.filter(item => item.id !== submission.id), submission],
        }))
        setNotice('Submitted for review.')
        setAttempt(current => current + 1)
    }

    const entries = [...(data?.milestones ?? [])].sort((a, b) => a.id - b.id).map(milestone => {
        const submissions = data.submissions.filter(submission => submission.milestone === milestone.id)
        return { milestone, submission: submissions.find(submission => submission.approved) ?? submissions[0] }
    })
    const showcase = entries.find(({ milestone }) => /^(final\s+)?showcase$/i.test(milestone.name.trim()))
    const milestones = entries.filter(entry => entry !== showcase)
    const approved = milestones.filter(entry => entry.submission?.approved).length
    const inReview = milestones.filter(entry => entry.submission && !entry.submission.approved).length

    return (
        <div className="haven-submission-stack">
            <section className="haven-team-card haven-submissions" aria-labelledby="submissions-title">
                <div className="haven-milestones-heading">
                    <h2 id="submissions-title">Milestones</h2>
                    {showcase && <a className="haven-submission-link" href="#showcase-title">Final Showcase ↓</a>}
                </div>
                <p role="status" className="haven-submission-notice">{notice}</p>
                {error && (
                    <div className="haven-submission-actions">
                        <p role="alert" className="haven-team-error">{error}</p>
                        <button type="button" className="haven-team-secondary" onClick={() => {
                            setError('')
                            setAttempt(current => current + 1)
                        }}>Try again</button>
                    </div>
                )}
                {!data && !error && <p role="status">Loading milestones…</p>}
                {data?.milestones.length === 0 && <p>No milestones have been published yet.</p>}
                {milestones.length > 0 && (
                    <>
                        <div className="haven-milestone-progress">
                            <p>{approved} of {milestones.length} approved{inReview > 0 && ` · ${inReview} in review`}</p>
                            <div className="haven-milestone-meter" aria-hidden="true">
                                {milestones.map(({ milestone, submission }) => (
                                    <span key={milestone.id} data-status={milestoneStatus(submission)} />
                                ))}
                            </div>
                        </div>
                        <ol className="haven-milestone-list" role="list">
                            {milestones.map((entry, index) => (
                                <MilestoneRow key={entry.milestone.id} entry={entry} number={index + 1}
                                    onSubmitted={submissionCreated} />
                            ))}
                        </ol>
                    </>
                )}
            </section>
            {showcase && (
                <section className="haven-team-card haven-showcase" aria-labelledby="showcase-title">
                    <div className="haven-showcase-heading">
                        <h2 id="showcase-title" ref={showcaseHeading} tabIndex={-1}>Final Showcase</h2>
                        <SubmissionAction milestone={showcase.milestone} submission={showcase.submission}
                            onSubmitted={submissionCreated} headingRef={showcaseHeading} />
                    </div>
                    <MilestoneDetails milestone={showcase.milestone} submission={showcase.submission} />
                </section>
            )}
        </div>
    )
}
