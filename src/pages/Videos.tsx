import { useEffect, useState } from "react";
import "../styles/Videos.css";
import { invoke } from "@tauri-apps/api/core";


type ErrorKind = {
	kind: 'databaseErr' | 'irohErr' | 'inputErr';
	message: string;
};

type Videos = Record<string, string[]>;

function VideosPage() {
	const [videos, setVideos] = useState<Videos>()
	const [error, setError] = useState<string | null>(null)

	const renderContent = () => {
		if (videos === undefined) return <div>
			Loading!
		</div>

		const videoEntries = Object.entries(videos);

		if (videoEntries.length === 0) return <div>
			No videos found, go add some?
		</div>

		return <div>
			{videoEntries.map(([namespace, filenames]) => (
				<div key={namespace}>
					<h2>{namespace}</h2>
					<ul>
						{filenames.map((filename) => <li key={filename}>{filename}</li>)}
					</ul>
				</div>
			))}
		</div>
	}

	useEffect(() => {
		invoke<Videos>('request_authorized_videos')
		.then((videos) => setVideos(videos))
		.catch((e: ErrorKind) => setError(e.message))
	}, [])

	return (
		<div>
			<h1>Videos</h1>

			{ error ? 
				<p style={{ color: "red", margin: "0 0 1em 0" }}>{error}</p> :
				renderContent()
			}

			
		</div>
	);
}

export default VideosPage;
