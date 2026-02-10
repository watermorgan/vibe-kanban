import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type {
  BaseCodingAgent,
  ExecutorConfig,
  ExecutorProfileId,
} from 'shared/types';
import { getVariantOptions } from '@/utils/executor';
import { useVariant } from './useVariant';

interface UseExecutorSelectionOptions {
  profiles: Record<string, ExecutorConfig> | null;
  latestProfileId: ExecutorProfileId | null;
  scratchProfileId: ExecutorProfileId | null | undefined;
  /** User's saved executor preference from config */
  configExecutorProfile?: ExecutorProfileId | null;
  /** Whether the user can override the executor via UI */
  allowUserSelection?: boolean;
  /** Optional key to reset local selection when switching context */
  scopeKey?: string;
}

interface UseExecutorSelectionResult {
  /** Effective executor: user selection > latest from processes > first available */
  effectiveExecutor: BaseCodingAgent | null;
  /** Available executor options */
  executorOptions: BaseCodingAgent[];
  /** Handle executor change (resets variant) */
  handleExecutorChange: (executor: BaseCodingAgent) => void;
  /** Currently selected variant */
  selectedVariant: string | null;
  /** Available variant options for current executor */
  variantOptions: string[];
  /** Set selected variant */
  setSelectedVariant: (variant: string | null) => void;
}

/**
 * Hook to manage executor and variant selection with priority:
 * - Executor: (optional) user selection > scratch > latest from processes > config > first available
 * - Variant: user selection > scratch > process
 */
export function useExecutorSelection({
  profiles,
  latestProfileId,
  scratchProfileId,
  configExecutorProfile,
  allowUserSelection = true,
  scopeKey,
}: UseExecutorSelectionOptions): UseExecutorSelectionResult {
  const [selectedExecutor, setSelectedExecutor] =
    useState<BaseCodingAgent | null>(null);
  const lastScopeKeyRef = useRef<string | undefined>(scopeKey);

  const executorOptions = useMemo(
    () => Object.keys(profiles ?? {}) as BaseCodingAgent[],
    [profiles]
  );

  useEffect(() => {
    const scopeChanged = lastScopeKeyRef.current !== scopeKey;
    if (!scopeChanged) return;
    lastScopeKeyRef.current = scopeKey;
    setSelectedExecutor(null);
  }, [scopeKey]);

  const effectiveExecutor = useMemo(
    () =>
      (allowUserSelection ? selectedExecutor : null) ??
      scratchProfileId?.executor ??
      latestProfileId?.executor ??
      configExecutorProfile?.executor ??
      executorOptions[0] ??
      null,
    [
      selectedExecutor,
      allowUserSelection,
      scratchProfileId?.executor,
      latestProfileId?.executor,
      configExecutorProfile?.executor,
      executorOptions,
    ]
  );

  const variantOptions = useMemo(
    () => getVariantOptions(effectiveExecutor, profiles),
    [effectiveExecutor, profiles]
  );

  const { selectedVariant, setSelectedVariant } = useVariant({
    processVariant: latestProfileId?.variant ?? null,
    scratchVariant: scratchProfileId?.variant,
    scopeKey,
  });

  const handleExecutorChange = useCallback(
    (executor: BaseCodingAgent) => {
      setSelectedExecutor(executor);
      // Reset variant to first available for the new executor
      const newVariantOptions = getVariantOptions(executor, profiles);
      setSelectedVariant(newVariantOptions[0] ?? null);
    },
    [profiles, setSelectedVariant]
  );

  return {
    effectiveExecutor,
    executorOptions,
    handleExecutorChange,
    selectedVariant,
    variantOptions,
    setSelectedVariant,
  };
}
