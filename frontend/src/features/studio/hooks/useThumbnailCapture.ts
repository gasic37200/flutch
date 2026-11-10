import { useEffect } from 'react'

const INTERVAL_MS = 60_000
const MAX_BYTES = 512 * 1024

/** 영상 한 프레임을 640x360 JPEG로. 크기가 한도를 넘으면 품질을 낮춰 다시 굽는다. */
async function snapshot(stream: MediaStream): Promise<Blob | null> {
  const track = stream.getVideoTracks()[0]
  if (!track || track.readyState !== 'live') return null
  const canvas = document.createElement('canvas')
  canvas.width = 640
  canvas.height = 360
  const g = canvas.getContext('2d')
  if (!g) return null
  g.fillStyle = '#000'
  g.fillRect(0, 0, 640, 360)

  // 탭이 백그라운드여도 프레임을 얻으려고 ImageCapture(Chromium)를 먼저 쓰고, 없으면 숨은 <video>로 읽는다.
  let src: CanvasImageSource, w: number, h: number
  if ('ImageCapture' in window) {
    const frame = await new (window as unknown as { ImageCapture: new (t: MediaStreamTrack) => { grabFrame(): Promise<ImageBitmap> } }).ImageCapture(track).grabFrame()
    src = frame
    w = frame.width
    h = frame.height
  } else {
    const video = document.createElement('video')
    video.muted = true
    video.playsInline = true
    video.srcObject = stream
    await video.play()
    src = video
    w = video.videoWidth
    h = video.videoHeight
    video.srcObject = null
  }
  if (!w || !h) return null
  const scale = Math.min(640 / w, 360 / h) // 비율을 유지해 가운데에 맞춘다
  g.drawImage(src, (640 - w * scale) / 2, (360 - h * scale) / 2, w * scale, h * scale)

  for (const quality of [0.8, 0.6, 0.4]) {
    const blob = await new Promise<Blob | null>((r) => canvas.toBlob(r, 'image/jpeg', quality))
    if (blob && blob.size <= MAX_BYTES) return blob
  }
  return null
}

/**
 * 송출 중 미리보기 영상을 1분마다 썸네일로 올린다(송출 시작 직후 1회 포함).
 * 카메라 전환 등으로 stream이 바뀌면 타이머를 다시 시작해(즉시 새 캡처) 새 stream을 쓴다. 실패는 조용히 다음 주기로 미룬다.
 */
export function useThumbnailCapture(broadcastId: number | undefined, live: boolean, stream: MediaStream | null) {
  useEffect(() => {
    if (!live || !stream || broadcastId === undefined) return
    let stopped = false
    const run = async () => {
      try {
        const s = stream
        const blob = s && (await snapshot(s))
        if (!blob || stopped) return
        await fetch(`/api/v1/broadcasts/${broadcastId}/thumbnail`, { method: 'PUT', credentials: 'include', headers: { 'Content-Type': 'image/jpeg' }, body: blob })
      } catch {
        // 다음 주기에 다시 시도한다
      }
    }
    const first = setTimeout(() => void run(), 3000) // 송출 직후 영상이 안정될 때까지 잠깐 기다린다
    const timer = setInterval(() => void run(), INTERVAL_MS)
    return () => {
      stopped = true
      clearTimeout(first)
      clearInterval(timer)
    }
  }, [broadcastId, live, stream])
}
