import Panel from '../components/ui/Panel.tsx'

type Section = { title: string; body: string[] }

const DOCS = {
  terms: {
    title: '이용약관',
    sections: [
      { title: '1. 서비스 성격', body: ['FLUTCH는 공모전 시연용으로 운영하는 라이브 스트리밍 서비스예요. 실제 상용 서비스가 아니며 예고 없이 중단·초기화될 수 있어요.'] },
      { title: '2. 계정', body: ['아이디는 가입 후 바꿀 수 없고, 탈퇴한 아이디는 다시 쓸 수 없어요.', '만 14세 이상만 가입할 수 있어요. 계정 관리 책임은 본인에게 있어요.'] },
      { title: '3. 비트', body: ['비트는 서비스 안에서만 쓰는 가상 재화예요. 결제는 테스트 모드로만 동작하며 실제 금액이 청구되지 않아요.', '탈퇴하면 남은 비트는 사라지고 되돌릴 수 없어요.'] },
      { title: '4. 금지 행위', body: ['타인에 대한 욕설·비방, 불법 콘텐츠 송출, 서비스 운영을 방해하는 행위는 금지돼요. 위반 시 채팅 제한·방송 숨김 등 조치를 받을 수 있어요.'] },
      { title: '5. 방송과 녹화', body: ['방송자는 자신의 방송에 대한 책임을 지며, 녹화를 켠 방송은 다시보기로 일정 기간 보관돼요.'] },
    ],
  },
  privacy: {
    title: '개인정보처리방침',
    sections: [
      { title: '1. 수집 항목', body: ['아이디, 비밀번호(복호화할 수 없는 해시로만 저장), 닉네임을 수집해요. 이메일·휴대폰 번호는 받지 않아요.', '이 서비스는 공모전 시연용이에요.'] },
      { title: '2. 보관 기간', body: [
        'IP 주소는 가입·로그인 요청 제한 계산에만 메모리에서 쓰고 저장하지 않아요.',
        '만료된 세션은 매일 정리해요.',
        '서버 로그는 7일, DB 백업은 7일 보관해요.',
      ] },
      { title: '3. 탈퇴 후', body: ['탈퇴하면 로그인할 수 없고 세션이 모두 만료돼요. 닉네임은 30일 후 다른 사람이 쓸 수 있게 풀려요.', '채팅·후원·비트 원장 기록은 서비스 정합성을 위해 "탈퇴한 사용자"로 바뀌어 보존돼요.'] },
      { title: '4. 제3자 제공', body: ['방송 송출·시청은 LiveKit Cloud, 결제 테스트는 토스페이먼츠 테스트 모드를 이용해요. 그 외 제3자에게 개인정보를 제공하지 않아요.'] },
    ],
  },
} satisfies Record<string, { title: string; sections: Section[] }>

export default function LegalPage({ doc }: { doc: keyof typeof DOCS }) {
  const { title, sections } = DOCS[doc]
  return (
    <div className='max-w-2xl'>
      <Panel innerClassName='flex flex-col gap-3 p-5 text-[13px] leading-[1.7] text-text-muted'>
        <h1 className='text-lg leading-[31px] font-bold text-text'>{title}</h1>
        <p className='chamfer bg-surface-2 px-3 py-2 text-text [--c:6px]'>공모전 시연용 서비스예요. 실제 결제는 일어나지 않아요.</p>
        {sections.map((s) => (
          <section key={s.title} className='flex flex-col gap-3'>
            <h2 className='font-bold text-text'>{s.title}</h2>
            {s.body.map((p) => (
              <p key={p}>
                {p}
              </p>
            ))}
          </section>
        ))}
        <span className='text-text-dim'>시행일 2026-11-19</span>
      </Panel>
    </div>
  )
}
