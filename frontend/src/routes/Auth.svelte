<script>
    import { wrt_username } from '../store';
    import { wrt_jwt } from '../store';
    /*
    let jwt;
    const unsubscribe_jwt = wrt_jwt.subscribe((value) => {
        jwt = value;
    });
    */

    function handleLogIn() {
        wrt_jwt.set(1);
    }
    function handleLogOut() {
        wrt_jwt.set(0);
    }
</script>


<div>
{#if $wrt_jwt > 0}
    <div>
        Hi {$wrt_username}
        <input type="submit" name="logout" value="LogOut" on:click={handleLogOut} />
    </div>	
    {:else}
    <div>
        Username:
        <input type="text" name="id" maxlength="30" bind:value={$wrt_username} />
        Password:
        <input type="text" name="pw" maxlength="30" value='' />
    
        <input type="submit" name="submit" value="Login" on:click={handleLogIn} />
    </div>
    {/if}
</div>

<style>
    input {
		 display: inline;
	}
</style>