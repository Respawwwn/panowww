<script lang="ts">
// Calibrate (spec §11.2): the four numbered tools and Step once arrive
// with issue 16; this is the card with its countdown line and the
// permanent Nothing moved? link.
import Tip from "../../components/Tip.svelte";
import { strings } from "../../lib/strings";
</script>

<section class="card calibrate">
  <h2 class="t-section">{strings.setup.calibrate.title}</h2>
  <Tip text={strings.setup.calibrate.countdown} />
  <!-- Tools 1 to 4 and Step once: issue 16. -->
  <a href="#nothing-moved" class="link" onclick={(e) => e.preventDefault()}>{strings.setup.calibrate.nothingMoved.link}</a>
</section>

<style>
  .calibrate {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px;
  }

  h2 {
    margin: 0;
  }

  .link {
    font-size: 12px;
    font-weight: 500;
    margin-top: auto;
  }
</style>
