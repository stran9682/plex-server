import "./Videos.css";

function VideoCard({ title }: { title: string }) {
	return <li>{title}</li>;
}

function VideoRow({ title, videos }: { title: string; videos: string[] }) {
	return (
		<div>
			<h2>{title}</h2>

			<ol>
				{videos.map((video, index) => (
					<VideoCard title={video} key={index} />
				))}
			</ol>
		</div>
	);
}

function VideosPage() {
	return (
		<div>
			<VideoRow title="Place" videos={["hi", "hey", "hello"]}></VideoRow>
			<VideoRow title="Place" videos={["hi", "hey", "hello"]}></VideoRow>
		</div>
	);
}

export default VideosPage;
