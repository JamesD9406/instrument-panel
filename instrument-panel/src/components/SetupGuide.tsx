interface Props {
  onClose: () => void
}

export function SetupGuide({ onClose }: Props) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>HWiNFO Setup Guide</h2>
          <button className="close-button" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="modal-body">
          <h3>1. Enable Shared Memory Support</h3>
          <ol>
            <li>Open HWiNFO</li>
            <li>
              Go to <strong>Settings</strong> (gear icon)
            </li>
            <li>
              Navigate to the <strong>General / User Interface</strong> tab
            </li>
            <li>
              Check <strong>"Shared Memory Support"</strong>
            </li>
            <li>Click OK and restart HWiNFO</li>
          </ol>

          <h3>2. Configure Sensors-only Mode (Recommended)</h3>
          <ol>
            <li>
              When HWiNFO starts, select <strong>"Sensors-only"</strong>
            </li>
            <li>This reduces overhead and focuses on monitoring</li>
          </ol>

          <h3>3. Optional: Start Minimized</h3>
          <ol>
            <li>In Settings → General / User Interface</li>
            <li>
              Check <strong>"Minimize Sensors on Startup"</strong>
            </li>
            <li>
              Check <strong>"Minimize Main Window on Startup"</strong>
            </li>
          </ol>

          <h3>⚠️ Important Note</h3>
          <p>
            The free version of HWiNFO has a{" "}
            <strong>12-hour shared memory limit</strong>. After 12 hours, you'll
            need to restart HWiNFO or purchase HWiNFO Pro to remove this
            limitation.
          </p>
          <p>
            When shared memory times out, The Instrument Panel will show "Not
            Connected". Simply restart HWiNFO to restore the connection.
          </p>
        </div>

        <div className="modal-footer">
          <button onClick={onClose}>Got it</button>
        </div>
      </div>
    </div>
  )
}
