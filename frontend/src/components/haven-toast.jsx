import { createPortal } from 'react-dom'
import mascot from '../assets/haven/hedgehog.webp'
import './haven-toast.css'

export default function HavenToast({ notice, onDismiss }) {
    if (!notice) return null

    const { title, message, tone = 'info' } = notice
    const symbol = { success: '✓', error: '!', info: 'i' }[tone]
    return createPortal(
        <div className="haven-toast-region">
            <div className="haven-toast" data-tone={tone}>
                <span className="haven-toast-mark" aria-hidden="true">{symbol}</span>
                <div className="haven-toast-copy">
                    <p className="haven-toast-title">{title}</p>
                    {message && <p>{message}</p>}
                </div>
                <img className="haven-toast-mascot" src={mascot} alt="" />
                <button type="button" className="haven-toast-close" onClick={onDismiss}
                    aria-label="Dismiss notification">×</button>
            </div>
        </div>,
        document.body,
    )
}
