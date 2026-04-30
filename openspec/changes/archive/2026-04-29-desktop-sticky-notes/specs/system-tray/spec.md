## ADDED Requirements

### Requirement: App minimizes to system tray on window close
The system SHALL intercept the window close event and hide the window instead of destroying it. The app SHALL continue running in the background with a system tray icon visible.

#### Scenario: User clicks the window close button
- **WHEN** user clicks the window close (X) button
- **THEN** the window is hidden (not destroyed) and the app remains accessible via the system tray icon
- **AND** the system tray icon remains visible in the OS taskbar/menu bar area

#### Scenario: Window is hidden and user clicks tray icon
- **WHEN** the window is hidden and user clicks the tray icon
- **THEN** the window is shown and brought to the foreground

### Requirement: System tray icon is displayed
The system SHALL display a tray icon in the operating system's system tray / menu bar area when the application is running. The icon SHALL be visually recognizable and consistent with the app's identity.

#### Scenario: App starts with tray icon
- **WHEN** the application launches
- **THEN** a tray icon appears in the system tray area

### Requirement: Tray context menu with actions
The system SHALL provide a right-click context menu on the tray icon with the following items:
1. "Show/Hide" — toggles main window visibility
2. "New Note" — shows the window and opens the sticky notes tab with focus on the create form
3. Separator
4. "Quit" — fully exits the application

#### Scenario: Show/Hide toggle via tray menu
- **WHEN** user right-clicks the tray icon and selects "Show/Hide"
- **THEN** if the window is visible it is hidden, or if hidden it is shown

#### Scenario: New Note from tray menu
- **WHEN** user right-clicks the tray icon and selects "New Note"
- **THEN** the main window is shown, the Sticky Notes tab is activated, and the note creation form receives focus

#### Scenario: Quit from tray menu
- **WHEN** user right-clicks the tray icon and selects "Quit"
- **THEN** the application exits completely, closing the window and removing the tray icon

### Requirement: Tray icon click toggles window visibility
The system SHALL toggle the main window's visibility when the user left-clicks (or primary-clicks) the tray icon.

#### Scenario: Click tray icon when window is visible
- **WHEN** the main window is visible and user clicks the tray icon
- **THEN** the main window is hidden

#### Scenario: Click tray icon when window is hidden
- **WHEN** the main window is hidden and user clicks the tray icon
- **THEN** the main window is shown and brought to the foreground

### Requirement: Cross-platform tray support
The system SHALL support system tray functionality on macOS, Windows, and Linux. Platform-specific behaviors (e.g., macOS menu bar icon vs Windows notification area) SHALL be handled by Tauri's native tray API.

#### Scenario: Tray icon on macOS
- **WHEN** the app runs on macOS
- **THEN** the tray icon appears in the macOS menu bar

#### Scenario: Tray icon on Windows
- **WHEN** the app runs on Windows
- **THEN** the tray icon appears in the Windows notification area (system tray)
