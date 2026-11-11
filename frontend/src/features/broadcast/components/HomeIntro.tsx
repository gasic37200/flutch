import Button from '../../../components/ui/Button.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import Panel from '../../../components/ui/Panel.tsx'

/** 홈에 라이브가 하나도 없을 때 히어로 자리에 서비스를 소개한다(심사 첫인상). */
export default function HomeIntro() {
  return (
    <Panel tone='hero' innerClassName='flex flex-col items-center gap-6 bg-surface p-6 text-center sm:p-10 md:flex-row md:text-left'>
      <img src='/brand/mascot-controller.png' alt='' className='h-40 w-auto shrink-0 object-contain md:h-56' />
      <div className='flex min-w-0 flex-col items-center gap-3 md:items-start'>
        <p className='flex items-center gap-2 font-display text-sm font-bold tracking-wider text-brand'>
          <Icon name='bolt' className='size-4' />
          지금은 조용하지만, 곧 터져요
        </p>
        <h2 className='font-display text-3xl font-bold text-text italic sm:text-4xl'>Feel the Moment. Live on FLUTCH.</h2>
        <p className='max-w-xl text-[15px] text-text-muted'>
          FLUTCH는 게임·버추얼 라이브 스트리밍 플랫폼이에요. 시청자가 CLUTCH 버튼을 함께 눌러 게이지를 채우면, 그 순간이 모두의 화면에서 터져요.
        </p>
        <div className='mt-1 flex flex-wrap justify-center gap-3 md:justify-start'>
          <Button to='/broadcasts/new'>방송하기</Button>
          <Button to='/explore' variant='secondary'>
            둘러보기
          </Button>
        </div>
      </div>
    </Panel>
  )
}
