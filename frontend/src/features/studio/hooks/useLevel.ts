import { useEffect, useState } from 'react'

/** AnalyserNode의 현재 음량(0~1)을 약 20fps로 돌려준다. 이 훅을 쓰는 작은 컴포넌트만 다시 그려진다. */
export function useLevel(analyser: AnalyserNode | null): number {
  const [level, setLevel] = useState(0)
  useEffect(() => {
    if (!analyser) return
    const buf = new Uint8Array(analyser.fftSize)
    const id = setInterval(() => {
      analyser.getByteTimeDomainData(buf)
      let sum = 0
      for (const v of buf) sum += ((v - 128) / 128) ** 2
      // RMS는 작아서 눈에 보이게 키운다
      setLevel(Math.min(1, Math.sqrt(sum / buf.length) * 3))
    }, 50)
    return () => {
      clearInterval(id)
      setLevel(0)
    }
  }, [analyser])
  return level
}
