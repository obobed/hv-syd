import { useNavigate } from 'react-router'
import Cookies from 'js-cookie'
import { useEffect, useState } from 'react'

export default function Dashboard() {
    const [isAdmin, setIsAdmin] = useState(false)
    const [hasTeam, setHasTeam] = useState(false) 
    const [submissions, setSubmissions] = useState([])

    useEffect(() => {
        const token = Cookies.get('admin-token');
        if (token) {
            setIsAdmin(true)
        }
    })

    useEffect(() => {
        if (isAdmin) {
            // list of all submissions ever
        } else {
            // list of all submissions (for a specific team)
        }
    })

    useEffect(() => {
        // get notifications somehow
    })

    async function sortSubmissions() {
        for (let i = 0; i < submissions.length; i++) {
            const currentSubmission = submissions[i]
            currentSubmission = JSON.parse(currentSubmission)
        }
    }

    return(
        <>
        <button onClick={() => setIsAdmin(!isAdmin)}>
            {isAdmin ? 'Switch to Attendee' : 'Switch to Admin'}
        </button>

        {isAdmin && (
            <>
                <div>
                    <p>unreviewed submissions</p>
                </div>
                <div>
                    <p>tokens</p>
                </div>
            </>
        )}

        {!isAdmin && (
            <>
                <div>
                    <p>list of submissions made + button to see all milestones</p>
                </div>
                <div>
                    <p>team name + member names</p>
                </div>
            </>
        )}
        </>
    )
}