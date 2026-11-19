import { useState, type ReactNode } from 'react'
import ChatCard from '../../chat/components/ChatCard.tsx'
import Button from '../../../components/ui/Button.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import { CATEGORY_LABEL } from '../../broadcast/categories.ts'
import Panel from '../../../components/ui/Panel.tsx'
import DonationQueueCard from './DonationQueueCard.tsx'
import StudioInfoForm from './StudioInfoForm.tsx'
import type { useStudio } from '../hooks/useStudio.ts'

type S = ReturnType<typeof useStudio>

function Card({ title, children }: { title: string; children: ReactNode }) {
  return (
    <Panel innerClassName='flex flex-col gap-2.5 p-4'>
      <h2 className='text-[15px] font-bold'>{title}</h2>
      {children}
    </Panel>
  )
}

// off = 꺼 둔 항목: 빈 원("아직 안 됨") 대신 대시로 구분한다
function Check({ done, off, children }: { done: boolean; off?: boolean; children: ReactNode }) {
  return (
    <li className={`flex items-center gap-2 ${done ? 'text-brand' : 'text-text-muted'}`}>
      {done ? <Icon name='check' className='size-4' /> : off ? <span className='ml-0.5 h-px w-3 bg-current' aria-hidden /> : <span className='ml-0.5 size-3 rounded-full border border-current' aria-hidden />}
      {children}
    </li>
  )
}

// 시안 Studio: 방송 정보 요약 한 줄 + "수정"으로 기존 폼을 펼친다
function InfoCard({ broadcast }: { broadcast: NonNullable<S['data']> }) {
  const [open, setOpen] = useState(false)
  return (
    <Panel className='shrink-0' innerClassName='flex flex-col gap-3 px-3.5 py-2.5'>
      <div className='flex items-center gap-2.5'>
        <div className='flex min-w-0 grow flex-col gap-0.5'>
          <span className='text-xs text-text-dim'>방송 정보</span>
          <b className='truncate text-[13px]'>{broadcast.title}</b>
          <span className='truncate text-xs text-text-muted'>{[CATEGORY_LABEL[broadcast.category], ...broadcast.tags].join(' · ')}</span>
        </div>
        <Button size='sm' variant='neutral' aria-expanded={open} aria-controls='studio-info-form' onClick={() => setOpen(!open)}>
          수정 {open ? '▴' : '▾'}
        </Button>
      </div>
      {open && (
        <div id='studio-info-form'>
          <StudioInfoForm broadcast={broadcast} onSaved={() => setOpen(false)} />
        </div>
      )}
    </Panel>
  )
}

// 송출 전: 시작 전 확인 · 권한 안내 · 지원 환경. 방송 중(시안 Studio): 채팅(남은 높이) · 후원 알림 대기열 · 방송 정보 요약.
export default function StudioAside({ s, live }: { s: S; live: boolean }) {
  const { data, media } = s
  if (!data) return null
  const screen = media.mode === 'screen'
  if (!live)
    return (
      <>
        <Card title='시작 전 확인'>
          <ul className='space-y-2 text-sm'>
            <Check done>로그인됨 · 내 방송</Check>
            <Check done={media.micReady}>{media.micReady ? '마이크 권한 허용' : '마이크 켜기 전'}</Check>
            <Check done={media.videoReady}>
              {screen ? (media.videoReady ? '공유할 화면 선택됨' : '공유할 화면 선택 전') : media.videoReady ? '카메라 켜짐' : '카메라 켜기 전'}
            </Check>
            <Check done={data.record} off={!data.record}>{data.record ? '녹화 켜짐 (최대 3시간)' : '녹화 꺼짐'}</Check>
          </ul>
        </Card>
        <Card title='권한이 막혔을 때'>
          <p className='text-[13px] leading-relaxed text-text-muted'>
            주소창 왼쪽 자물쇠 아이콘 → 사이트 설정 → 카메라·마이크를 &quot;허용&quot;으로 바꾼 뒤 새로고침해 주세요. 화면 공유와 마이크는 HTTPS에서만 동작해요.
          </p>
        </Card>
        <Card title='지원 환경'>
          <p className='text-[13px] leading-relaxed text-text-muted'>
            시스템 소리: Windows Chrome·Edge
            <br />
            탭 소리: 데스크톱 Chrome·Edge
            <br />
            모바일: 카메라 송출 (Android Chrome, iOS Safari)
          </p>
        </Card>
      </>
    )
  return (
    <>
      <ChatCard chat={s.chat} broadcast={data} className='h-[340px] lg:h-auto lg:min-h-0 lg:flex-1' />
      <DonationQueueCard alerts={s.alerts} />
      <InfoCard broadcast={data} />
    </>
  )
}
