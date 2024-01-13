import { createResource, createSignal } from "solid-js";
import solidLogo from "./assets/solid.svg";
import viteLogo from "/vite.svg";
import "./App.css";
// https://web.dev/articles/passkey-registration

async function isPassKeysSupported() {
  if (
    !window.PublicKeyCredential ||
    !PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable ||
    !PublicKeyCredential.isConditionalMediationAvailable
  )
    return false;

  const results = await Promise.all([
    PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable(),
    PublicKeyCredential.isConditionalMediationAvailable(),
  ]);

  // Can they return non boolean truthy values?
  return results[0] === true && results[1] === true;
}

const encoder = new TextEncoder();

async function getCredentials() {
  const response = await fetch("http://localhost:3000/credentials");

  const json = await response.json();

  // Replace url encoding
  json.challenge = json.challenge.replace(/-/g, "+").replace(/_/g, "/");
  json.user.id = json.user.id.replace(/-/g, "+").replace(/_/g, "/");

  // Decode base64
  json.challenge = atob(json.challenge);
  json.user.id = atob(json.user.id);

  // Then to Uint8Array...
  json.challenge = encoder.encode(json.challenge);
  json.user.id = encoder.encode(json.user.id);

  return json;
}

function App() {
  const [count, setCount] = createSignal(0);
  const [isSupported] = createResource(isPassKeysSupported);

  const [credentials] = createResource(getCredentials);

  async function createCredential() {
    const creationOptions = {
      ...credentials(),
      rp: {
        name: "localhost page",
        id: "localhost",
      },
      pubKeyCredParams: [
        { alg: -7, type: "public-key" },
        // { alg: -257, type: "public-key" },
      ],
      authenticatorSelection: {
        authenticatorAttachment: "platform",
        requireResidentKey: true,
        userVerification: "required",
      },
    };

    console.debug({ publicKey: creationOptions });
    const credential = await navigator.credentials.create({
      publicKey: creationOptions,
    });
    console.debug(credential);
  }
  return (
    <>
      {isSupported.loading
        ? "Loading..."
        : isSupported()
        ? "Is supported"
        : "Not supported"}

      <p>
        If you're using Bitwarden this might fail on localhost http debugging.
        You need to disable the extension temporarily. It's a{" "}
        <a href="https://github.com/bitwarden/clients/issues/6882">
          known issue
        </a>
        .
      </p>
      {!credentials.loading && (
        <button onClick={createCredential}>Create Credential</button>
      )}
    </>
  );
}

export default App;
