import type { DebuggerProtocolEnvelope, MockMessageSet } from "./protocol";

// Purpose: Holds the minimal reducer state for the live debugger transport without introducing external state libraries.

export type LiveDebuggerMode = "loading" | "live" | "fallback";

export type LiveDebuggerState = {
  mode: LiveDebuggerMode;
  messages: DebuggerProtocolEnvelope[];
  loading: boolean;
  submitting: boolean;
  errorMessage: string;
};

export type LiveDebuggerAction =
  | { type: "loadStarted" }
  | { type: "loadSucceeded"; messages: DebuggerProtocolEnvelope[] }
  | { type: "loadFellBack"; messages: DebuggerProtocolEnvelope[]; errorMessage: string }
  | { type: "submitStarted" }
  | { type: "submitSucceeded"; messages: DebuggerProtocolEnvelope[] }
  | { type: "submitFailed"; errorMessage: string };

export function createInitialLiveDebuggerState(
  fallbackMessageSets: MockMessageSet[]
): LiveDebuggerState {
  return {
    mode: "loading",
    messages: fallbackMessageSets[0]?.messages ?? [],
    loading: true,
    submitting: false,
    errorMessage: ""
  };
}

export function liveDebuggerReducer(
  state: LiveDebuggerState,
  action: LiveDebuggerAction
): LiveDebuggerState {
  switch (action.type) {
    case "loadStarted":
      return {
        ...state,
        loading: true,
        errorMessage: ""
      };
    case "loadSucceeded":
      return {
        mode: "live",
        messages: action.messages,
        loading: false,
        submitting: false,
        errorMessage: ""
      };
    case "loadFellBack":
      return {
        mode: "fallback",
        messages: action.messages,
        loading: false,
        submitting: false,
        errorMessage: action.errorMessage
      };
    case "submitStarted":
      return {
        ...state,
        submitting: true,
        errorMessage: ""
      };
    case "submitSucceeded":
      return {
        ...state,
        messages: [...state.messages, ...action.messages],
        submitting: false
      };
    case "submitFailed":
      return {
        ...state,
        submitting: false,
        errorMessage: action.errorMessage
      };
    default:
      return state;
  }
}
