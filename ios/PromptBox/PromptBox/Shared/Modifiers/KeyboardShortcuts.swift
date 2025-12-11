import SwiftUI

// iPad keyboard shortcuts support
extension View {
    @ViewBuilder
    func keyboardShortcut(_ key: KeyEquivalent, modifiers: EventModifiers = [], action: @escaping () -> Void) -> some View {
        self.background {
            Button("") {
                action()
            }
            .keyboardShortcut(key, modifiers: modifiers)
            .opacity(0)
        }
    }
}

// Common keyboard shortcuts for the app
struct KeyboardShortcutsModifier: ViewModifier {
    let onRate: (Int) -> Void
    let onFavorite: () -> Void
    let onPrevious: () -> Void
    let onNext: () -> Void
    let onDelete: () -> Void

    func body(content: Content) -> some View {
        content
            .keyboardShortcut("1", modifiers: []) { onRate(1) }
            .keyboardShortcut("2", modifiers: []) { onRate(2) }
            .keyboardShortcut("3", modifiers: []) { onRate(3) }
            .keyboardShortcut("4", modifiers: []) { onRate(4) }
            .keyboardShortcut("5", modifiers: []) { onRate(5) }
            .keyboardShortcut("0", modifiers: []) { onRate(0) }
            .keyboardShortcut("f", modifiers: []) { onFavorite() }
            .keyboardShortcut(.leftArrow, modifiers: []) { onPrevious() }
            .keyboardShortcut(.rightArrow, modifiers: []) { onNext() }
            .keyboardShortcut(.delete, modifiers: .command) { onDelete() }
    }
}

extension View {
    func imageKeyboardShortcuts(
        onRate: @escaping (Int) -> Void,
        onFavorite: @escaping () -> Void,
        onPrevious: @escaping () -> Void,
        onNext: @escaping () -> Void,
        onDelete: @escaping () -> Void
    ) -> some View {
        modifier(KeyboardShortcutsModifier(
            onRate: onRate,
            onFavorite: onFavorite,
            onPrevious: onPrevious,
            onNext: onNext,
            onDelete: onDelete
        ))
    }
}
