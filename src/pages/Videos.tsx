import { useEffect, useState } from "react";
import "../styles/Videos.css";
import { invoke } from "@tauri-apps/api/core";

type ErrorKind = {
	kind: "databaseErr" | "irohErr" | "inputErr";
	message: string;
};

interface VideoInfo {
	tag: string;
	videoName: string;
}

type Videos = Record<string, VideoInfo[]>;

const VideoPlayer = ({selectedVideo} : {selectedVideo:string}) => {
	const [namespace, resource] = selectedVideo.split("/")

	useEffect(() => {
		invoke('start_adding_topic_peers', {namespace: namespace})
		.catch((e) => console.error(e))

		return () => {
			invoke('stop_adding_topic_peers', {namespace: namespace})
			.catch((e) => console.error(e))
		};
	}, [selectedVideo])

	const playlist = `http://127.0.0.1:3000/video/${encodeURIComponent(namespace)}/${encodeURIComponent(resource)}/playlist.m3u8`

	return <video
		src={playlist}
		controls
		width="640"
		playsInline
	/>
}


function VideosPage() {
	const [videos, setVideos] = useState<Videos>()
	const [error, setError] = useState<string | null>(null);
	const [selectedVideo, setSelectedVideo] = useState<string | null>(null);

	const renderList = () => {
		if (videos === undefined) return <div>Loading!</div>

		const videoEntries = Object.entries(videos);

		if (videoEntries.length === 0)
			return <div>No videos found, go add some?</div>

		return (
			<div>
				{videoEntries.map(([namespace, filenames]) => (
					<div key={namespace}>
						<h2>{namespace.substring(0, 10)}</h2>
						<ol>
							{filenames.map((video) => (<>
									<li onClick = {() => setSelectedVideo(video.tag)} key={video.tag}>{video.videoName}</li>
									{video.tag}
								</>
							))}
						</ol>
					</div>
				))}
			</div>
		);
	};

	useEffect(() => {
		invoke<Videos>('request_authorized_videos')
		.then((videos) => setVideos(videos))
		.catch((e: ErrorKind) => setError(e.message))
	}, [])

	return (
		<div>
			<h1>Videos</h1>

			{selectedVideo && <VideoPlayer selectedVideo={selectedVideo}/>}

			{error ? (
				<p style={{ color: "red", margin: "0 0 1em 0" }}>{error}</p>
			) : (
				renderList()
			)}
		</div>
	);
}

export default VideosPage;
