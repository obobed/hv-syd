import { useEffect, useState } from 'react'
import { Link, useOutletContext } from 'react-router'
import TeamDashboard from './team-dashboard'

async function fetchTokens(signal) {
    const response = await fetch('/api/admin/token/list', { credentials: 'same-origin', signal })
    if (!response.ok) throw new Error(`HTTP error: ${response.status}`)
    return response.json()
}

export default function Dashboard() {
    const session = useOutletContext()
    if (session === undefined) return <p role="status">Checking your session…</p>
    if (!session) return <p><Link to="/">Sign in to view your dashboard</Link></p>
    if (session.role === 'staff') return <StaffDashboard />
    return <TeamDashboard />
}

function StaffDashboard() {
    const [tokens, setTokens] = useState({})
    const [ascending, setAscending] = useState(false)
    const [tokenSeconds, setTokenSeconds] = useState(0)
    const [tokenNanoseconds, setTokenNanoseconds] = useState(0)
    const [targetToken, setTargetToken] = useState('')

    const tokenLevel = {
        ADMIN: 'admin',
        ATTENDEE: 'attendee'
    }

    async function getTokens() {
        try {
            setTokens(await fetchTokens())
        } catch (error) {
            console.error('Couldn’t load tokens', error)
        }
    }

    useEffect(() => {
        const controller = new AbortController()
        fetchTokens(controller.signal).then(tokenList => {
            if (!controller.signal.aborted) setTokens(tokenList)
        }).catch(error => {
            if (!controller.signal.aborted) console.error('Couldn’t load tokens', error)
        })
        return () => controller.abort()
    }, [])

    async function createToken(url = '', payload = {}) {
        try {
            const response = await fetch(url, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                credentials: 'include',
                body: JSON.stringify(payload)
            })

            if (!response.ok) {
                throw new Error(`HTTP error: ${response.status}`);
            }

            await getTokens()
        } catch (error) {
            console.error('Couldn’t create token', error)
        }
    }

    async function tokenUpdate(url = '', payload = {}) {
        try {
            const response = await fetch(url, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                credentials: "include",
                body: JSON.stringify(payload)
            })

            if (!response.ok) {
                throw new Error (`HTTP error! Status: ${response.status}`)
            }

            await getTokens()

        } catch {
            console.log('error')
        }
    }

    return(
        <main>
            <div>
                <p>unreviewed submissions</p>
            </div>
            <div>
                <dialog id='tokenCreator'>
                    <button onClick={() =>
                        createToken('/api/admin/token/create',
                            {
                                level: tokenLevel.ADMIN,
                                valid_for: [Number(tokenSeconds), Number(tokenNanoseconds)]
                            }
                        )
                    }>
                        Create Token
                    </button>
                    <input
                        type='number'
                        placeholder='Seconds'
                        value={tokenSeconds}
                        onChange={(e) => setTokenSeconds(e.target.value)}
                    />
                    <input
                        type='number'
                        placeholder='Nanoseconds'
                        value={tokenNanoseconds}
                        onChange={(e) => setTokenNanoseconds(e.target.value)}
                    />
                    <button command='close' commandfor='tokenCreator'>
                        X
                    </button>
                </dialog>

                <button command='show-modal' commandfor='tokenCreator'>
                    Create New Token
                </button>

                <button onClick={() => void getTokens()}>
                    Refresh Tokens
                </button>

                <button onClick={() => setAscending(!ascending)}>
                    {ascending ? 'Expires First' : 'Expires Last'}
                </button>
                <table>
                    <thead>
                        <tr>
                            <th>Token ID</th>
                            <th>Expiry</th>
                        </tr>
                    </thead>
                    <tbody>
                        {Object.entries(tokens)
                            .sort(([, a], [, b]) =>
                                ascending
                                ? b.exp.localeCompare(a.exp)
                                : a.exp.localeCompare(b.exp)
                            )
                            .map(([tokenId, metadata]) => (
                            <tr key={tokenId}>
                                <td><code>{tokenId}</code></td>
                                <td>{new Date(metadata.exp).toLocaleString()}</td>
                                <td><button onClick={() => setTargetToken(tokenId)} command='show-modal' commandfor='tokenUpdator'>Edit</button></td>
                            </tr>
                        ))}
                    </tbody>
                </table>

                <dialog id='tokenUpdator'>
                    <p> Updating token {targetToken}</p>

                    <button onClick={() => tokenUpdate(`/api/admin/token/${targetToken}`, {
                        add: [Number(tokenSeconds), Number(tokenNanoseconds)]
                    })
                    } command='close' commandfor='tokenUpdator'>
                        Extend Token
                    </button>

                    <input
                        type='number'
                        placeholder='Seconds'
                        value={tokenSeconds}
                        onChange={(e) => setTokenSeconds(e.target.value)}
                    />
                    <input
                        type='number'
                        placeholder='Nanoseconds'
                        value={tokenNanoseconds}
                        onChange={(e) => setTokenNanoseconds(e.target.value)}
                    />
                    <button command='close' commandfor='tokenUpdator'>
                        X
                    </button>
                </dialog>
            </div>
        </main>
    )
}
