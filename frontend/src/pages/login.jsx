import { useEffect, useMemo, useRef, useState } from 'react'
import { useNavigate } from 'react-router'
import { Scanner, setZXingModuleOverrides } from '@yudiel/react-qr-scanner'
import scannerWasm from 'zxing-wasm/reader/zxing_reader.wasm?url'
import logo from '../assets/haven/logo.webp'
import HavenScene from '../components/haven-scene'
import { clearPreviousRole } from '../lib/session'
import './login.css'

// Serve the QR decoder from the app so scanning works without a CDN request.
setZXingModuleOverrides({ locateFile: path => path === 'zxing_reader.wasm' ? scannerWasm : path })

const ATTEND_ID = /^[\da-f]{8}-[\da-f]{4}-[\da-f]{4}-[\da-f]{4}-[\da-f]{12}$/i
const ATTEND_QR = 'attend://checkin/'
const QR_FORMATS = ['qr_code']
const SCANNER_COMPONENTS = { finder: false }
const SCANNER_STYLES = {
    container: { width: '100%', height: '100%', aspectRatio: 'auto' },
    video: { width: '100%', height: '100%', objectFit: 'cover' },
}
const UNAVAILABLE = 'Sign-in is unavailable right now. Try again in a moment.'

function readAttendId(value) {
    const code = value.trim().toLowerCase()
    const id = code.startsWith(ATTEND_QR) ? code.slice(ATTEND_QR.length) : code
    return ATTEND_ID.test(id) ? id : null
}

function requestError(status, staff) {
    if (status === 429) return 'Too many attempts. Wait a moment and try again.'
    if ([400, 401, 403, 404].includes(status)) {
        return staff
            ? 'Check your sign-in details and try again.'
            : 'We couldn’t verify that Attend ID. Check your ticket and try again.'
    }
    return UNAVAILABLE
}

function cameraError(kind) {
    switch (kind) {
        case 'permission-denied':
        case 'security':
            return 'Camera access was blocked. Allow it in your browser, or enter your Attend ID.'
        case 'no-camera':
            return 'No camera was found. You can enter your Attend ID instead.'
        case 'insecure-context':
        case 'unsupported':
            return 'The camera isn’t available here. You can enter your Attend ID instead.'
        case 'in-use':
            return 'The camera is busy. Close any other app using it and try again.'
        default:
            return 'Couldn’t open the scanner. Try again or enter your Attend ID.'
    }
}

function Icon({ name, size = 18 }) {
    const paths = {
        scan: 'M8 3H5a2 2 0 0 0-2 2v3m13-5h3a2 2 0 0 1 2 2v3M3 16v3a2 2 0 0 0 2 2h3m8 0h3a2 2 0 0 0 2-2v-3M7 7h3v3H7zm7 0h3v3h-3zm-7 7h3v3H7zm7 0h1m2 0v3h-3',
        frame: 'M8 3H5a2 2 0 0 0-2 2v3m13-5h3a2 2 0 0 1 2 2v3M3 16v3a2 2 0 0 0 2 2h3m8 0h3a2 2 0 0 0 2-2v-3',
        camera: 'M4 6h4l2-2h4l2 2h4a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2zm12 7a4 4 0 1 0-8 0 4 4 0 0 0 8 0z',
        arrow: 'm12 5 7 7-7 7M5 12h14',
    }
    return (
        <svg aria-hidden="true" width={size} height={size} viewBox="0 0 24 24"
            fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round">
            <path d={paths[name]} />
        </svg>
    )
}

// The official site's Hero.svelte logo lockup.
function HavenHeroBrand() {
    return (
        <div className="haven-hero-brand">
            <img src={logo} alt="Hack Club Haven" width="762" height="491" />
            <span>Sydney</span>
        </div>
    )
}

// SignupForm.svelte's orange CTA, adapted to an action or form submission.
function HavenAction({ buttonRef, children, ...props }) {
    return <button ref={buttonRef} className="haven-primary" {...props}>{children}</button>
}

function TicketScanner({ onScan, onError, onStop }) {
    const active = useRef(false)
    const [devices, setDevices] = useState([])
    const [device, setDevice] = useState('')
    const [ready, setReady] = useState(false)
    const constraints = useMemo(() => device
        ? { deviceId: { exact: device } }
        : { facingMode: { ideal: 'environment' } }, [device])

    useEffect(() => {
        active.current = true
        return () => { active.current = false }
    }, [])

    async function handlePlaying() {
        setReady(true)
        try {
            const cameras = await navigator.mediaDevices.enumerateDevices()
            if (active.current) setDevices(cameras.filter(camera => camera.kind === 'videoinput' && camera.deviceId))
        } catch {
            // The active camera can still be used if enumeration fails.
        }
    }

    return (
        <div className="haven-camera-live" onPlayingCapture={handlePlaying}>
            <Scanner constraints={constraints} formats={QR_FORMATS} sound={false}
                components={SCANNER_COMPONENTS} styles={SCANNER_STYLES}
                onScan={codes => { if (active.current) onScan(codes) }}
                onError={error => { if (active.current) onError(error) }}
                startTimeoutMs={10000} />
            <div className="haven-camera-guide" aria-hidden="true"><Icon name="frame" size={132} /></div>
            <span className="haven-visually-hidden" role="status">
                {ready ? 'Line up your QR code' : 'Starting camera…'}
            </span>
            <div className="haven-camera-controls">
                {devices.length > 1 && (
                    <select aria-label="Camera" value={device} onChange={event => {
                        setReady(false)
                        setDevice(event.target.value)
                    }}>
                        <option value="">Default camera</option>
                        {devices.map((camera, index) => (
                            <option key={camera.deviceId} value={camera.deviceId}>{camera.label || `Camera ${index + 1}`}</option>
                        ))}
                    </select>
                )}
                <button type="button" onClick={onStop} autoFocus>Stop camera</button>
            </div>
        </div>
    )
}

export default function Login() {
    const navigate = useNavigate()
    const request = useRef(null)
    const [mode, setMode] = useState('attendee')
    const [scanning, setScanning] = useState(false)
    const [cameraMessage, setCameraMessage] = useState('')
    const [attendId, setAttendId] = useState('')
    const [busy, setBusy] = useState(false)
    const [error, setError] = useState('')
    const cameraButton = useRef(null)
    const restoreCameraFocus = useRef(false)

    const staff = mode === 'staff' || mode === 'token'
    const instructions = staff ? 'Sign in to manage the event.' : 'Scan your QR or type in your ID from your Attend ticket.'

    useEffect(() => () => request.current?.abort(), [])
    useEffect(() => {
        if (mode === 'attendee' && !scanning && !busy && restoreCameraFocus.current) {
            cameraButton.current?.focus()
            restoreCameraFocus.current = false
        }
    }, [mode, scanning, busy])

    function stopCamera() {
        restoreCameraFocus.current = true
        setScanning(false)
    }

    function changeMode(next) {
        if (request.current) return
        setError('')
        setScanning(false)
        setCameraMessage('')
        restoreCameraFocus.current = next === 'attendee'
        setMode(next)
    }

    async function signIn(path, payload) {
        if (request.current) return

        const controller = new AbortController()
        request.current = controller
        setBusy(true)
        setError('')
        let timedOut = false
        let signedIn = false
        const timeout = setTimeout(() => {
            timedOut = true
            controller.abort()
        }, 10000)

        try {
            const response = await fetch(path, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                credentials: 'include',
                redirect: 'error',
                signal: controller.signal,
                body: JSON.stringify(payload),
            })

            if (response.headers.get('content-type')?.includes('text/html')) {
                setError(UNAVAILABLE)
            } else if (!response.ok) {
                setError(requestError(response.status, staff))
            } else {
                clearPreviousRole(staff)
                signedIn = true
                navigate('/dashboard', { replace: true })
            }
        } catch {
            if (timedOut) setError('Sign-in took too long. Try again.')
            else if (!controller.signal.aborted) setError(UNAVAILABLE)
        } finally {
            clearTimeout(timeout)
            if (!signedIn) {
                request.current = null
                if (timedOut || !controller.signal.aborted) setBusy(false)
            }
        }
    }

    function handleScan(codes) {
        if (request.current) return
        const id = codes.map(code => readAttendId(code.rawValue)).find(Boolean)
        if (!id) {
            setError('That isn’t an Attend ticket. Try the QR code on your Haven ticket.')
            return
        }

        setAttendId(id)
        stopCamera()
        void signIn('/api/v1/login', { id })
    }

    function submit(event) {
        event.preventDefault()
        const data = new FormData(event.currentTarget)
        if (staff) {
            const payload = mode === 'token'
                ? { token: String(data.get('token')).trim() }
                : { username: data.get('username'), password: data.get('password') }
            if (mode === 'token' && !payload.token) {
                setError('Enter your admin token.')
                return
            }
            void signIn('/api/admin/login', payload)
            return
        }

        const id = readAttendId(attendId)
        if (!id) {
            setError('Enter the Attend ID from your ticket.')
            return
        }
        setScanning(false)
        void signIn('/api/v1/login', { id })
    }

    return (
        <main className="haven-login">
            <HavenScene />
            <div className="haven-login-shell">
                <HavenHeroBrand />

                <section className="haven-login-panel" aria-labelledby="haven-login-title">
                    <header className="haven-login-heading">
                        <h1 id="haven-login-title">{staff ? 'Staff sign-in' : 'Sign in to Haven'}</h1>
                        <p>{instructions}</p>
                    </header>

                    {mode === 'attendee' ? (
                        <>
                            <div className="haven-camera">
                                {scanning ? (
                                    <TicketScanner onScan={handleScan} onStop={stopCamera}
                                        onError={failure => {
                                            setCameraMessage(cameraError(failure.kind))
                                            stopCamera()
                                        }} />
                                ) : (
                                    <div className="haven-camera-idle">
                                        {cameraMessage ? <p role="alert">{cameraMessage}</p> : <Icon name="scan" size={36} />}
                                        <HavenAction buttonRef={cameraButton} type="button" disabled={busy}
                                            onClick={() => {
                                                setError('')
                                                setCameraMessage('')
                                                setScanning(true)
                                            }}>
                                            <Icon name="camera" /> {cameraMessage ? 'Try again' : 'Scan QR'}
                                        </HavenAction>
                                    </div>
                                )}
                            </div>
                            <form className="haven-login-form haven-id-form" aria-busy={busy}
                                onSubmit={submit} onChange={() => setError('')}>
                                <label htmlFor="haven-attend-id">Attend ID</label>
                                <div className="haven-id-row">
                                    <input id="haven-attend-id" className="haven-input" name="id" placeholder="Paste your Attend ID"
                                        value={attendId} onChange={event => setAttendId(event.target.value)}
                                        autoComplete="off" autoCapitalize="none" spellCheck={false}
                                        enterKeyHint="go" readOnly={busy} required />
                                    <HavenAction type="submit" disabled={busy}>
                                        {busy ? 'Signing in…' : 'Sign in'}
                                    </HavenAction>
                                </div>
                            </form>
                        </>
                    ) : (
                        <>
                            <form key={mode} className="haven-login-form" aria-busy={busy}
                                onSubmit={submit} onChange={() => setError('')}>
                                {mode === 'token' ? (
                                    <label className="haven-field">
                                        Admin token
                                        <input className="haven-input" name="token" type="password" autoComplete="off" readOnly={busy} autoFocus required />
                                    </label>
                                ) : (
                                    <>
                                        <label className="haven-field">
                                            Username
                                            <input className="haven-input" name="username" autoComplete="username" readOnly={busy} autoFocus required />
                                        </label>
                                        <label className="haven-field">
                                            Password
                                            <input className="haven-input" name="password" type="password" autoComplete="current-password" readOnly={busy} required />
                                        </label>
                                    </>
                                )}
                                <HavenAction type="submit" disabled={busy}>
                                    {busy ? 'Signing in…' : 'Sign in'} {!busy && <Icon name="arrow" />}
                                </HavenAction>
                            </form>
                            <button className="haven-link haven-alternate" type="button" disabled={busy}
                                onClick={() => changeMode(mode === 'staff' ? 'token' : 'staff')}>
                                {mode === 'staff' ? 'Use an admin token' : 'Use username and password'}
                            </button>
                        </>
                    )}

                    {error && <p className="haven-login-error" role="alert">{error}</p>}
                    {busy && <span className="haven-visually-hidden" role="status">Signing in…</span>}

                    <footer className="haven-login-footer">
                        <button className="haven-link" type="button" disabled={busy}
                            onClick={() => changeMode(staff ? 'attendee' : 'staff')}>
                            {staff ? 'Back to attendee sign-in' : 'Staff sign-in'}
                        </button>
                    </footer>
                </section>
            </div>
        </main>
    )
}
