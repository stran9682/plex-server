import './Videos.css'

function VideoCard ({title}: {title: String}) {
    return <div className="video-card">
        {title}
    </div>
}

function VideoRow ({title, videos}: {title: string, videos: string[]} ) {
    return <div>
        <h2>{title}</h2>

        <div className='video-row'>
            {videos.map((video, index) => (
                <VideoCard title={video} key={index} />
            ))}
        </div>
        
    </div>
}

function VideosPage () {
    return <div>
        <VideoRow title="Place" videos={["hi", "hey", "hello"]}></VideoRow>    
        <VideoRow title="Place" videos={["hi", "hey", "hello"]}></VideoRow>    
    </div>
}

export default VideosPage