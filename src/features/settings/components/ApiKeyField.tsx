import * as React from 'react';
import { Eye, EyeSlash } from '@phosphor-icons/react';
import { cn } from '@/lib/utils';
import '../styles/api-key-field.css';

interface ApiKeyFieldProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'className' | 'type'> {
  revealed: boolean;
  canReveal: boolean;
  disabled?: boolean;
  showLabel: string;
  hideLabel: string;
  onToggle: () => void;
  inputClassName?: string;
  className?: string;
  /**
   * Optional slot for additional adornments (e.g. a copy button) rendered
   * inside the shell, to the left of the reveal toggle.
   * Use `.api-key-field__action` class on child buttons to match the
   * built-in toggle's styling.
   */
  extraActions?: React.ReactNode;
}

export const ApiKeyField = React.forwardRef<HTMLInputElement, ApiKeyFieldProps>(({
  revealed,
  canReveal,
  disabled,
  showLabel,
  hideLabel,
  onToggle,
  inputClassName,
  className,
  extraActions,
  onPaste: onPasteProp,
  onChange: onChangeProp,
  ...props
}, ref) => {
  const label = revealed ? hideLabel : showLabel;
  const inputType = canReveal && revealed ? 'text' : 'password';

  // React controlled <input type="password"> sometimes does NOT fire onChange
  // when the user pastes (Ctrl+V / right-click paste) in Tauri WebView2.
  // DispatchEvent(new InputEvent(...)) is unreliable because React ignores
  // non-trusted (isTrusted:false) events in some WebView2 builds.
  // Instead, we directly invoke the parent's onChange handler with the DOM
  // value after the browser has applied the paste — same pattern as
  // SecurePasswordInput.
  const handlePaste = React.useCallback((e: React.ClipboardEvent<HTMLInputElement>) => {
    onPasteProp?.(e);
    const input = e.currentTarget;
    // setTimeout(0) ensures the browser has written the pasted text to input.value
    setTimeout(() => {
      if (onChangeProp) {
        onChangeProp({ target: input } as React.ChangeEvent<HTMLInputElement>);
      }
    }, 0);
  }, [onPasteProp, onChangeProp]);

  return (
    <div
      data-api-key-field
      className={cn(
        'api-key-field',
        disabled && 'api-key-field--disabled',
        className
      )}
    >
      <input
        ref={ref}
        type={inputType}
        disabled={disabled}
        className={cn(
          'api-key-field__input',
          inputClassName
        )}
        onPaste={handlePaste}
        onChange={onChangeProp}
        {...props}
      />
      {extraActions}
      {canReveal && (
        // eslint-disable-next-line ds-components/no-native-button -- Input adornment needs exact height/edge control instead of shared button primitive sizing.
        <button
          type="button"
          onClick={onToggle}
          disabled={disabled}
          aria-label={label}
          aria-pressed={revealed}
          title={label}
          className="api-key-field__action api-key-field__toggle"
        >
          {revealed ? <Eye className="api-key-field__icon" /> : <EyeSlash className="api-key-field__icon" />}
        </button>
      )}
    </div>
  );
});

ApiKeyField.displayName = 'ApiKeyField';
