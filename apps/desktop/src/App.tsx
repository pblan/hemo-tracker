import {
  Alert,
  Box,
  Button,
  Field,
  Heading,
  Input,
  Stack,
  Text,
} from "@chakra-ui/react";
import { type FormEvent, useEffect, useState } from "react";

import {
  createLocalAccount,
  getVaultState,
  lockVault,
  unlockWithPassphrase,
  unlockWithRecovery,
  type VaultState,
} from "./vault-client";

function App() {
  const [vaultState, setVaultState] = useState<VaultState | null>(null);
  const [recoveryCode, setRecoveryCode] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void getVaultState()
      .then(setVaultState)
      .catch(() => {
        setError("Hemo Tracker could not read the local vault state.");
      });
  }, []);

  return (
    <Box
      as="main"
      minH="100vh"
      bg="bg"
      color="fg"
      px={{ base: "6", md: "10" }}
      py="12"
    >
      <Stack gap="8" maxW="3xl" mx="auto">
        <Stack gap="3">
          <Text
            color="teal.600"
            fontSize="sm"
            fontWeight="semibold"
            letterSpacing="wide"
            textTransform="uppercase"
          >
            Local laboratory tracking
          </Text>
          <Heading as="h1" size="4xl">
            Hemo Tracker
          </Heading>
          <Text color="fg.muted" fontSize="lg">
            Your laboratory data stays in an encrypted vault on this computer.
          </Text>
        </Stack>
        {error ? (
          <Alert.Root status="error">
            <Alert.Indicator />
            <Alert.Title>{error}</Alert.Title>
          </Alert.Root>
        ) : null}
        {!vaultState ? <Text>Read local vault…</Text> : null}
        {vaultState?.status === "missing" && !recoveryCode ? (
          <CreateVault
            onCreated={(code) => {
              setRecoveryCode(code);
              setVaultState({ accountExists: true, status: "unlocked" });
            }}
            onError={setError}
          />
        ) : null}
        {recoveryCode ? (
          <RecoveryKey
            code={recoveryCode}
            onConfirmed={() => setRecoveryCode(null)}
          />
        ) : null}
        {vaultState?.status === "locked" ? (
          <UnlockVault onUnlocked={setVaultState} onError={setError} />
        ) : null}
        {vaultState?.status === "unlocked" && !recoveryCode ? (
          <UnlockedVault onLocked={setVaultState} onError={setError} />
        ) : null}
      </Stack>
    </Box>
  );
}

function CreateVault({
  onCreated,
  onError,
}: {
  onCreated: (code: string) => void;
  onError: (message: string) => void;
}) {
  const [passphrase, setPassphrase] = useState("");
  const [confirmation, setConfirmation] = useState("");
  async function submit(event: FormEvent) {
    event.preventDefault();
    if (passphrase !== confirmation)
      return onError("The passphrases do not match.");
    try {
      const created = await createLocalAccount(passphrase);
      setPassphrase("");
      setConfirmation("");
      onCreated(created.recoveryCode);
    } catch {
      onError("Hemo Tracker could not create the local vault.");
    }
  }
  return (
    <Box borderWidth="1px" borderRadius="xl" p={{ base: "5", md: "7" }}>
      <Stack as="form" gap="5" onSubmit={submit}>
        <Stack gap="2">
          <Heading as="h2" size="xl">
            Create your local vault
          </Heading>
          <Text color="fg.muted">
            Use a unique passphrase. Hemo Tracker cannot recover it for you.
          </Text>
        </Stack>
        <Field.Root required>
          <Field.Label>Passphrase</Field.Label>
          <Input
            type="password"
            autoComplete="new-password"
            value={passphrase}
            onChange={(event) => setPassphrase(event.target.value)}
          />
        </Field.Root>
        <Field.Root required>
          <Field.Label>Confirm passphrase</Field.Label>
          <Input
            type="password"
            autoComplete="new-password"
            value={confirmation}
            onChange={(event) => setConfirmation(event.target.value)}
          />
        </Field.Root>
        <Button type="submit" alignSelf="start" colorPalette="teal">
          Create vault
        </Button>
      </Stack>
    </Box>
  );
}

function RecoveryKey({
  code,
  onConfirmed,
}: {
  code: string;
  onConfirmed: () => void;
}) {
  return (
    <Box borderWidth="1px" borderRadius="xl" p={{ base: "5", md: "7" }}>
      <Stack gap="5">
        <Heading as="h2" size="xl">
          Store your recovery key
        </Heading>
        <Text>
          Store this recovery key separately from this computer. You need it if
          you forget the passphrase.
        </Text>
        <Box
          as="code"
          overflowWrap="anywhere"
          bg="bg.muted"
          borderRadius="md"
          p="4"
        >
          {code}
        </Box>
        <Button alignSelf="start" onClick={onConfirmed}>
          I stored the recovery key
        </Button>
      </Stack>
    </Box>
  );
}

function UnlockVault({
  onUnlocked,
  onError,
}: {
  onUnlocked: (state: VaultState) => void;
  onError: (message: string) => void;
}) {
  const [passphrase, setPassphrase] = useState("");
  const [recoveryCode, setRecoveryCode] = useState("");
  async function withPassphrase(event: FormEvent) {
    event.preventDefault();
    onError("");
    try {
      onUnlocked(await unlockWithPassphrase(passphrase));
    } catch {
      onError("The passphrase or local vault is invalid.");
    } finally {
      setPassphrase("");
    }
  }
  async function withRecovery(event: FormEvent) {
    event.preventDefault();
    onError("");
    try {
      onUnlocked(await unlockWithRecovery(recoveryCode));
    } catch {
      onError("The recovery key or local vault is invalid.");
    } finally {
      setRecoveryCode("");
    }
  }
  return (
    <Stack gap="6">
      <Box
        as="form"
        borderWidth="1px"
        borderRadius="xl"
        p="6"
        onSubmit={withPassphrase}
      >
        <Stack gap="4">
          <Heading as="h2" size="xl">
            Unlock your vault
          </Heading>
          <Field.Root required>
            <Field.Label>Passphrase</Field.Label>
            <Input
              type="password"
              value={passphrase}
              onChange={(event) => setPassphrase(event.target.value)}
            />
          </Field.Root>
          <Button type="submit" alignSelf="start" colorPalette="teal">
            Unlock vault
          </Button>
        </Stack>
      </Box>
      <Box
        as="form"
        borderWidth="1px"
        borderRadius="xl"
        p="6"
        onSubmit={withRecovery}
      >
        <Stack gap="4">
          <Heading as="h2" size="lg">
            Use recovery key
          </Heading>
          <Field.Root required>
            <Field.Label>Recovery key</Field.Label>
            <Input
              value={recoveryCode}
              onChange={(event) => setRecoveryCode(event.target.value)}
            />
          </Field.Root>
          <Button type="submit" alignSelf="start" variant="outline">
            Unlock with recovery key
          </Button>
        </Stack>
      </Box>
    </Stack>
  );
}

function UnlockedVault({
  onLocked,
  onError,
}: {
  onLocked: (state: VaultState) => void;
  onError: (message: string) => void;
}) {
  async function lock() {
    try {
      onLocked(await lockVault());
    } catch {
      onError("Hemo Tracker could not lock the local vault.");
    }
  }
  return (
    <Box borderWidth="1px" borderRadius="xl" p="7">
      <Stack gap="4">
        <Heading as="h2" size="xl">
          Your vault is unlocked
        </Heading>
        <Text color="fg.muted">You can now record your first lab report.</Text>
        <Button alignSelf="start" variant="outline" onClick={() => void lock()}>
          Lock vault
        </Button>
      </Stack>
    </Box>
  );
}

export default App;
