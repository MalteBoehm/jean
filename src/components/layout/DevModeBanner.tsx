export function DevModeBanner() {
  if (!import.meta.env.DEV) {
    return null
  }

  return (
    <div
      data-tauri-drag-region
      className="relative flex h-6 w-full shrink-0 items-center justify-center overflow-hidden bg-yellow-400"
    >
      {/* Left diagonal stripes */}
      <div
        className="absolute left-0 top-0 h-full w-16"
        style={{
          background:
            'repeating-linear-gradient(135deg, #000 0px, #000 8px, #facc15 8px, #facc15 16px)',
        }}
      />
      {/* Right diagonal stripes */}
      <div
        className="absolute right-0 top-0 h-full w-16"
        style={{
          background:
            'repeating-linear-gradient(135deg, #000 0px, #000 8px, #facc15 8px, #facc15 16px)',
        }}
      />
      {/* Text */}
      <span className="z-10 text-xs font-bold tracking-wider text-black">
        DEVELOPMENT MODE
      </span>
    </div>
  )
}
