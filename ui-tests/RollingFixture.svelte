<script lang="ts">
  import RollingDigit from "../src/lib/components/time/RollingDigit.svelte";
  let value = $state("0");
  (window as any).setDigit = (next: string) => value = next;
</script>
<RollingDigit {value}/>
