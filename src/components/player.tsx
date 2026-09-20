import '@videojs/react/video/skin.css';
import { VideoPlayer, VideoSkin } from '@videojs/react/video';
import { HlsJsVideo } from '@videojs/react/media/hlsjs-video';

export const Player = ({ src }: {src: string}) => {
  return (
    <VideoPlayer>
      <VideoSkin>
        <HlsJsVideo src={src} playsInline />
      </VideoSkin>
    </VideoPlayer>
  );
};