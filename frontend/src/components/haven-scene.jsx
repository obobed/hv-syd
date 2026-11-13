import mascot from '../assets/haven/hedgehog.webp'
import foreground from '../assets/haven/foreground.webp'
import './haven-scene.css'

export default function HavenScene() {
    return (
        <div className="haven-scene" aria-hidden="true">
            <img className="haven-scene-mascot" src={mascot} alt="" width="422" height="308" />
            <img className="haven-scene-foreground" src={foreground} alt="" width="1414" height="501" />
        </div>
    )
}
