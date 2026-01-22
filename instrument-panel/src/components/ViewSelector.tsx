import { useState, useRef, useEffect } from "react"

interface View {
  id: string
  label: string
}

interface Props {
  views: View[]
  activeView: string
  onViewChange: (viewId: string) => void
}

export function ViewSelector({ views, activeView, onViewChange }: Props) {
  const [isOpen, setIsOpen] = useState(false)
  const dropdownRef = useRef<HTMLDivElement>(null)

  const activeLabel =
    views.find((v) => v.id === activeView)?.label ?? "Overview"

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false)
      }
    }
    document.addEventListener("mousedown", handleClickOutside)
    return () => document.removeEventListener("mousedown", handleClickOutside)
  }, [])

  return (
    <div className="view-selector" ref={dropdownRef}>
      <button
        className="view-selector-toggle"
        onClick={() => setIsOpen(!isOpen)}
      >
        <span>{activeLabel}</span>
        <span className="view-selector-arrow">{isOpen ? "▲" : "▼"}</span>
      </button>

      {isOpen && (
        <div className="view-selector-dropdown">
          {views.map((view) => (
            <button
              key={view.id}
              className={`view-selector-option ${activeView === view.id ? "active" : ""}`}
              onClick={() => {
                onViewChange(view.id)
                setIsOpen(false)
              }}
            >
              {view.label}
            </button>
          ))}
        </div>
      )}
    </div>
  )
}
