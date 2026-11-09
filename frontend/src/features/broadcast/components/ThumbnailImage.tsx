import { faceSrc } from '../../../components/ui/mascot.ts'

// 썸네일이 없으면 배경 텍스처 + 카테고리 + 졸린 고양이(LiveCard와 같은 기본 이미지)
export default function ThumbnailImage({ src, category }: { src: string | null; category: string }) {
  if (src) return <img src={src} alt='' loading='lazy' className='size-full object-cover' />
  return (
    <div className='relative flex size-full items-center justify-center bg-surface-2 bg-cover bg-center' style={{ backgroundImage: 'url(/brand/bg-speed.jpg)' }}>
      <span className='font-display font-bold tracking-wide text-text-muted'>{category}</span>
      <img src={faceSrc('sleepy')} alt='' className='absolute right-1 bottom-1 size-6' />
    </div>
  )
}
