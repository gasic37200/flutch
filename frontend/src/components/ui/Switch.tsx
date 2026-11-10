type Props = { label: string; checked: boolean; onChange: (next: boolean) => void; disabled?: boolean }

export default function Switch({ label, checked, onChange, disabled }: Props) {
  return (
    <button
      type='button'
      role='switch'
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={`relative h-7 w-12 shrink-0 rounded-full transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${checked ? 'bg-brand' : 'bg-border'}`}
    >
      <span className={`absolute top-[3px] left-[3px] size-[22px] rounded-full transition-transform ${checked ? 'translate-x-5 bg-bg' : 'bg-text-dim'}`} />
    </button>
  )
}
