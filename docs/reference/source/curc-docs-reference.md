# CU Research Computing (CURC) - REFERENCE

**Source**: https://curc.readthedocs.io/en/latest/

> This file contains the original CURC documentation dump used to create [`alpine-cluster-reference.md`](../alpine-cluster-reference.md). For current Alpine cluster information, always use [`alpine-cluster-reference.md`](../alpine-cluster-reference.md) instead, where relevant information has been extracted into a structured cluster guide. 
> This file is preserved for any additional future reference needs.

# Frequently Asked Questions

## General High Performance Computing

### How do I check how full my directories are?
::::{dropdown} Show 
:icon: note

You have three directories allocated to your username (`$USER`). These include `/home/$USER` (2 G), `/projects/$USER` (250 G) and `/scratch/alpine/$USER` (10 T).  To see how much space you've used in each, from a login node, type `curc-quota` as follows:

```
[janedoe@login11 ~]$ curc-quota
------------------------------------------------------------------------
									Used         Avail    Quota Limit
------------------------------------------------------------------------
/home/janedoe                          1.7G          339M           2.0G
/projects/janedoe                       67G          184G           250G
/scratch/alpine1                      1050G         8950G         10000G
```

You can also check the amount of space being used by any directory with the `du -sh` command or the directory's contents with the `du -h` command: 

```
[janedoe@c3cpu-a7-u26-3 ~]$ du -h /scratch/alpine/janedoe/WRF
698M	WRF/run
698M	WRF
```
::::

### When will my job start?
::::{dropdown} Show 
:icon: note

You can pull up information on your job's start time using the `squeue` command: 
```
squeue --user=your_rc-username --start
```
Note that Slurm's estimated start time can be a bit inaccurate. This is because Slurm calculates this estimation off the jobs that are currently running or queued in the system. Any job that is added in later with a higher priority may delay your job.

For more information on the `squeue` command, [take a look at our Useful Slurm Commands tutorial.](../running-jobs/slurm-commands.md) Or visit the Slurm page on [squeue](https://slurm.schedmd.com/squeue.html)

::::

### How much memory did my job use?
::::{dropdown} Show 
:icon: note

You can check how much memory your job utilized by using the `sacct` command and refering to the `MaxRSS` metric. This is done as follows where you can replace `YYYY-MM-DD` with the date you ran the job and specify your JobID:

```
sacct --starttime=YYYY-MM-DD --jobs=your_job_id --format=User,JobName,JobId,MaxRSS
```

If you'd like to monitor memory usage on jobs that are currently running, use the `sstat` command:

```
sstat --jobs=your_job_id --format=User,JobName,JobId,MaxRSS
```

For more information on `sstat` or `sacct` commands, [take a look at our Useful Slurm Commands tutorial.](../running-jobs/slurm-commands.md) Or visit the Slurm reference pages on [sstat](https://slurm.schedmd.com/sstat.html) and [sacct](https://slurm.schedmd.com/sacct.html). 

You can also view information related to service unit (SU) usage and CPU & RAM efficiency by using [slurmtools](../compute/monitoring-resources.md#slurmtools). Note that CPU & RAM efficiency statistics will be included in emails sent when a job completes, if requested. 
::::

### How can I see my current FairShare priority?
::::{dropdown} Show 
:icon: note

There are a couple ways you can check your FairShare priority:

1. Using the `levelfs` tool in the `slurmtools` module. `levelfs` shows the current fair share priority of a specified user.
	
	You can use this tool by first loading in the `slurmtools` module (available from login nodes):
	```
	$ module load slurmtools
	```

	```{tip}
	slurmtools is packed with lots of great features and tools like suacct, suuser, jobstats, seff, etc._
	```

	Then using `levelfs` on your username:
	```
	$ levelfs $USER
	```
	* A value of 1 indicates average priority compared to other users in an account.
	* A value of < 1 indicates lower than average priority (longer than average queue waits) 
	* A value of > 1 indicates higher than average priority (shorter than average queue waits)
<br/><br/>
2. Using the `sshare` command:
	```
	sshare -U -l
	```
	The `sshare` command will print out a table of information regarding your usage and priority on all allocations. The `-U` flag will specify the current user and the `-l` flag will print out more details in the table. The field we are looking for is the _LevelFS_. The LevelFS holds a number from 0 to infinity that describes the fair share of an association in relation to its other siblings in an account. Over-serviced accounts will have a LevelFS between 0 and 1. Under-serviced accounts will have a LevelFS greater than 1. Accounts that haven't run any jobs will have a LevelFS of infinity (inf).

	For more information on fair share the `sshare` command, [take a look at Slurm's documentation on fair share](https://slurm.schedmd.com/fair_tree.html) Or [check out the Slurm reference page on sshare](https://slurm.schedmd.com/sshare.html)
::::

### Why is my job pending with reason `ReqNodeNotAvail`?
::::{dropdown} Show 
:icon: note

The `ReqNodeNotAvail` message usually means that your node has been reserved for maintenance during the period you have requested within your job script. This message often occurs in the days leading up to our regularly scheduled maintenance, which is performed the first Wednesday of every month. So, for example, if you run a job with a 72 hour wall clock request on the first Monday of the month, you will receive the `ReqNodeNotAvail` error because the node is reserved for maintenance within that 72-hour window. You can confirm whether the requested node has a reservation by typing `scontrol show reservation` to list all active reservations. 

If you receive this message, the following solutions are available: 
1. Run a shorter job or modify your current job's time so that it does not intersect with the maintenance window. One can modify your current job's time by using the `scontrol` command:

	```bash
	$ scontrol update jobid=<jobid> time=<time>
	```
2. Wait until after maintenance window has finished. Once maintenance has completed, your job will resume automatically.
::::

### How can I check what accounts (allocations) I belong to?
::::{dropdown} Show 
:icon: note

You can check the allocations you belong to with the `sacctmgr` command. This can be done by typing the following from a login or compute node:
```bash
sacctmgr -p show associations user=$USER
```
This will print out an assortment of information including allocations and QoS available to you. For more information on sacctmgr, please refer to [Slurm's documentation](https://slurm.schedmd.com/sacctmgr.html). 
::::

### Why do I get an `LMOD` error when I try to load Slurm?
::::{dropdown} Show 
:icon: note
The `slurm/alpine` and `slurm/blanca` module environments cannot be loaded from compute nodes. It should only be loaded from login nodes when attempting to switch between Blanca and Alpine environments. This error can be disregarded, as no harm is done.
::::

## Alpine 

### Why do I get an `Invalid Partition` error when running an Alpine job?
::::{dropdown} Show 
:icon: note

This error usually means users do not have an allocation that would provide the service units (SUs) required to run a job.  This can occur if a user has no valid allocation, specifies an invalid allocation, or specifies an invalid partition.  Think of SUs as "HPC currency": you need an allocation of SUs to use the system. Allocations are free. New CU users should automatically get added to a `ucb-general` allocation upon account creation which will provide a modest allocation of SUs for running small jobs and testing/benchmarking codes. However, if this allocation expires and you do not have a new one you will see this error.  `ucb-general` allocations are intended for benchmarking and testing and it is expected that users will move to a project allocation.  To request a Project and apply for a Project Allocation visit our [allocation documentation](../clusters/alpine/allocations.md).
::::

# Alpine Hardware

## Hardware Summary

```{important}
All Alpine nodes are available to all users. For full details about node access, please read the [Alpine node access and FairShare policy](condo-fairshare-and-resource-access.md).
```

### University of Colorado Boulder contribution

:::{table}
:width: 95%
:widths: auto
:align: left


| Count & Type          | Partition | Processor        | Sockets | Cores (total) | Threads per Core | RAM per Core (GB) | L3 Cache (MB) | GPU type    | GPU count | Local Disk Capacity & Type | Fabric                                       | OS       |
| --------------------- | ------------------- | ---------------- | :-------: | :-------------: | :------------: | :-------------: | :-------------: | ----------- | :---------: | -------------------------- | -------------------------------------------- | -------- |
| {{ alpine_ucb_total_64_core_256GB_cpu_nodes }} Milan General CPU | amilan              | x86_64 AMD Milan | 1 or 2  | 64            | 1            |  3.8          | 32            | N/A         | 0         | 416G SSD                   | HDR-100 InfiniBand (200Gb inter-node fabric) | RHEL 8.4 |
| {{ alpine_ucb_total_128_core_256GB_cpu_nodes }} Milan CPU | amilan128c             | x86_64 AMD Milan | 2  | 128            | 1            |  2.01         | 32            | N/A         | 0         | 416G SSD | HDR-100 InfiniBand (200Gb inter-node fabric) | RHEL 8.8 |
| {{ alpine_ucb_total_48_core_1TB_cpu_nodes }} Milan High-Memory  | amem                | x86_64 AMD Milan | 2       | 48            | 1            | 21.5          | 32            | N/A         | 0         | 416G SSD                   | 2x25 Gb Ethernet +RoCE                       | RHEL 8.4 |
| {{ alpine_ucb_total_64_core_1TB_cpu_nodes }} Milan High-Memory   | amem                | x86_64 AMD Milan | 1       | 64            | 1            |  16           | 32            | N/A         | 0         | 416G SSD                   | 2x25 Gb Ethernet +RoCE                       | RHEL 8.4 |
| {{ alpine_ucb_total_mi100_gpu_nodes }} Milan AMD GPU | ami100              | x86_64 AMD Milan | 2       | 64            | 1            |  3.8          | 32            | AMD MI100   | 3         | 416G SSD                   | 2x25 Gb Ethernet +RoCE                       | RHEL 8.4 |
| {{ alpine_ucb_total_a100_gpu_nodes }} Milan NVIDIA GPU    | aa100               | x86_64 AMD Milan | 2       | 64            | 1            |  3.8          | 32            | NVIDIA A100 | 3         | 416G SSD                   | 2x25 Gb Ethernet +RoCE                       | RHEL 8.4 |
| {{ alpine_ucb_total_acompile_nodes }} Milan CPU compile nodes | acompile | x86_64 AMD Milan | 1 or 2  | 64            | 1            |  3.8          | 32            | N/A         | 0         | 416G SSD                   | HDR-100 InfiniBand (200Gb inter-node fabric) | RHEL 8.4 |
| {{ alpine_ucb_total_64_core_256GB_cpu_nodes_atesting }} Milan CPU test nodes; pulls from CU amilan pool | atesting | x86_64 AMD Milan | 1 or 2  | 64            | 1            |  3.8          | 32            | N/A         | 0         | 416G SSD                   | HDR-100 InfiniBand (200Gb inter-node fabric) | RHEL 8.4 |
| {{ alpine_ucb_total_atesting_a100_gpu_nodes }} Milan NVIDIA GPU testing node | atesting_a100 | x86_64 AMD Milan | 2       | 64            | 1            |  3.8          | 32            | NVIDIA A100 | 3 (each split by MIG)        | 416G SSD                   | 2x25 Gb Ethernet +RoCE                       | RHEL 8.4 |
| {{ alpine_ucb_total_atesting_mi100_gpu_nodes }} Milan AMD GPU testing nodes; pulls from ami100 pool | atesting_mi100 | x86_64 AMD Milan | 2       | 64            | 1            |  3.8          | 32            | AMD MI100   | 3         | 416G SSD                   | 2x25 Gb Ethernet +RoCE                       | RHEL 8.4 |

:::

## Requesting Hardware Resources
Resources are requested within jobs by passing in SLURM directives, or resource flags, to either a job script (most common) or to the command line when submitting a job. Below are some common resource directives for Alpine (summarized then detailed):
* **Gres (General Resources):** Specifies the number of GPUs (*required if using a GPU node*)
* **QOS (Quality of Service):** Constrains or modifies job characteristics
* **Partition:** Specifies node type

### General Resources (gres)

**General resources allows for fine-grain hardware specifications**. On Alpine the `gres` directive is _**required**_ to use GPU accelerators on GPU nodes. At a minimum, one would specify `--gres=gpu` in their job script (or on the command line when submitting a job) to specify that they would like to use a single GPU on their specified partition. One can also request multiple GPU accelerators on nodes that have multiple accelerators. Alpine GPU resources and configurations can be viewed as follows on a login node with the `slurm/alpine` module loaded:

```bash
$ sinfo --Format Partition,Gres |grep gpu |grep -v -i aa100
```

__Examples of GPU configurations/requests__:

(tabset-ref-ex-gpu-conf-req)=
`````{tab-set}
:sync-group: tabset-ex-gpu-conf-req

````{tab-item} Single GPU
:sync: ex-gpu-conf-req-single-gpu

**Request a single GPU accelerator.**

```bash
--gres=gpu
```

````

```` {tab-item} Multiple GPUs
:sync: ex-gpu-conf-req-multiple-gpu

**Request multiple (in this case 3) GPU accelerators.**

```bash
--gres=gpu:3
```

````

`````

### Quality of Service (qos)

**Quality of Service or QoS is used to constrain or modify the characteristics that a job can have.** For example, by selecting the `long` QoS, a user can place the job in a **lower priority queue** with a max wall time increased from 24 hours to 7 days.

The available QoS for Alpine:

| QOS name    | Description                | Max walltime    | Max jobs/user | Node limits        | Valid Partitions | 
| ----------- | -------------------------- | --------------- | ------------- | ------------------ | ---------------- |
| normal      | Standard QoS for non-testing partitions                    | 1 day              | 1000          | 128                | amilan,amilan128c,aa100,ami100  |
| long        | Longer wall times          | 7 days              | 200           | 20                | amilan,amilan128c,aa100,ami100              | 
| mem         | High-memory jobs           | 7 days              | 1000          | 12                 | amem only        | 
| testing         | Used for all testing partitions   | 1 hour              | 5          |  2       | atesting,atesting_a100,atesting_mi100     | 
| compile       | Used for acompile jobs  | 12 hours              |    -      |   1      | acompile   | 

__QoS examples__:

(tabset-ref-ex-qos-req)=
`````{tab-set}
:sync-group: tabset-ex-qos-req

````{tab-item} Requesting the normal partition 
:sync: ex-qos-req-normal-partition

```bash
--qos=normal
```

````

```` {tab-item} Requesting the long partition
:sync: ex-qos-req-long-partition

```bash
--qos=long
```

````

`````


### Partitions

**Nodes with the same hardware configuration are grouped into partitions**. You specify a partition using the `--partition` SLURM directive in your job script (or at the command line when submitting an interactive job) in order for your job to run on the appropriate type of node. 

```{note}
GPU nodes require the additional `--gres` directive (see above section).
```

Partitions available on Alpine:


| Partition | Description                  | # of nodes | cores/node | RAM/core (GB) | Billing_weight/core | Default/Max Walltime     | Resource Limits |
| --------- | ---------------------------- | ---------- | ---------- | ------------- | ------------------- | ------------------------ | ----------------------|
| amilan    | AMD Milan (default)          | {{ alpine_total_amilan_nodes }}        | 32 or 48 or 64 |   3.75        | 1                   | 24H, 7D                 | see qos table |
| amilan128c    | AMD Milan        | {{ alpine_total_amilan128c_nodes }}        | 128 |   2.01        | 1                   | 24H, 7D      | see qos table |
| ami100    | GPU-enabled (3x AMD MI100)   | {{ alpine_total_ami100_nodes }}          | 64         |   3.75        | 6.1<sup>3</sup>     | 24H, 7D                 | 15 GPUs across all jobs |
| aa100     | GPU-enabled (3x NVIDIA A100)<sup>4</sup> | {{ alpine_total_aa100_nodes }}          | 64         |   3.75       | 6.1<sup>3</sup>     | 24H, 7D     | 21 GPUs across all jobs |
| al40      | GPU-enabled (3x NVIDIA L40)<sup>4</sup> | {{ alpine_total_al40_nodes }}          | 64         |   3.75       | 6.1<sup>3</sup>     | 24H, 7D     | 6 GPUs across all jobs |
| amem<sup>1</sup> | High-memory           | {{ alpine_total_amem_nodes }}          | 48 or 64 or 128     |  16<sup>2</sup> | 4.0           |  4H,  7D                 | 128 cores across all jobs |
| acompile | AMD Milan compile nodes | {{ alpine_total_acompile_nodes }} | 64 |   3.75        | N/A                   | see [acompile section](./alpine-hardware.md#acompile-usage-examples) below                 | see [acompile section](./alpine-hardware.md#atesting-usage-examples) below |
| atesting | AMD Milan test nodes | {{ alpine_total_atesting_cpu_nodes }}; Pulls from CU amilan pool | 64 |   3.75        | 0.025                   | see [atesting section](./alpine-hardware.md#atesting-usage-examples) below                 | see [atesting section](./alpine-hardware.md#atesting-usage-examples) below |
| atesting_a100 | GPU-enabled testing node (3x NVIDIA A100 split w/ MIG) | {{ alpine_total_atesting_a100_nodes }} | 64         |   3.75       | 0.025     | see [GPU atesting section](./alpine-hardware.md#gpu-atesting-usage-examples) below     | see [GPU atesting section](./alpine-hardware.md#gpu-atesting-usage-examples) below |
| atesting_mi100 | GPU-enabled testing nodes (3x AMD MI100) | {{ alpine_total_atesting_mi100_nodes }} | 64         |   3.75       | 0.025     | see [GPU atesting section](./alpine-hardware.md#gpu-atesting-usage-examples) below     | see [GPU atesting section](./alpine-hardware.md#gpu-atesting-usage-examples) below |

```{important}
**Partition table footnotes:** 


<sup>1</sup>The `amem` partition requires the mem QOS. The mem QOS is only available to jobs asking for 256GB of RAM or more, 12 nodes or fewer, and 128 cores or fewer. For example, you can run one 128-core job or up to two 64-core jobs, etc.

<sup>2</sup>The `amem` partition has a mixture of nodes with 48, 64, and 128 cores.  Nodes with 48 and 64 cores have 1 TB of RAM; nodes with 128 cores have 2 TB of RAM.  The default RAM-per-requested core on the `amem` partition is 15,927 MB, which is configured such that if you request all 64 (128) cores on a 64-core (128-core) `amem` node, you will receive roughly 1,000,000 MB of RAM (i.e., the full ~1 TB available). If you request all 48 cores on a 48-core node, by default you will receive 764,496 MB of RAM, which is less than the 1 TB available. If you require more RAM than the default of 15,927 MB per-requested-core, employ the `--mem` flag in your job script and specify the amount of RAM you need, in MB. For example, to request all of the RAM on a node, use "--mem=1000000M".   

<sup>3</sup>On the GPU partitions, `ami100`, `aa100`, and `al40`, the _billing_weight_ value of 6.1/core is an aggregate estimate. In practice, users are billed 1.0 for each core they request, and 108.2 for each GPU they request. For example, if a user requests all 64 cores and all three GPUs for one hour, they will be billed (1.0 * 64) + (108.2 * 3)=389 SUs.

<sup>4</sup>NVIDIA A100 and L40 GPUs only support CUDA versions >11.x
```

All users, regardless of institution, should specify partitions as follows:
```bash
--partition=amilan
--partition=amilan128c
--partition=aa100
--partition=ami100
--partition=al40
--partition=amem
```

#### Special-Purpose Partitions

To help users test out their workflows, CURC provides several special-purpose partitions on Alpine. These partitions enable users to quickly test or compile code on CPU and GPU compute nodes. To ensure equal access to these special-purpose partitions, the amount of resources (such as CPUs, GPUs, and runtime) are limited. 

```{important}
Compiling and testing partitions are, as their name implies, only meant for compiling code and testing workflows. They are not to be used outside of compiling or testing. Please utilize the appropriate partitions when running code. 
```

##### `atesting` usage examples:

`atesting` provides access to limited resources for the purpose of verifying workflows and MPI jobs. Users are able to request up to 2 CPU nodes (8 cores per node) for a maximum runtime of 1 hour (default  1 hour) and 16 CPUs. Users who need GPU nodes to test workflows should use the appropriate GPU testing partitions (`atesting_a100` or `atesting_mi100`) instead of `atesting`.

(tabset-ref-atesting-use)=
`````{tab-set}
:sync-group: tabset-atesting-use

````{tab-item} Example 1
:sync: atesting-use-ex1

**Request one core per node for 10 minutes.**

```bash
sinteractive --partition=atesting --ntasks-per-node=1 --nodes=2 --time=00:10:00
```

````

```` {tab-item} Example 2
:sync: atesting-use-ex2

**Request 4 cores for the default time of 30 minutes.**

```bash
sinteractive --partition=atesting --ntasks=4
```

````

```` {tab-item} Example 3
:sync: atesting-use-ex3

**Request 2 cores each from 2 nodes for testing MPI.**

```bash
sinteractive --ntasks-per-node=2 --nodes=2 --partition=atesting
```

````
`````

##### GPU `atesting` usage examples:

`atesting_a100` and `atesting_mi100` provide access to limited GPU resources for the purpose of verifying GPU workflows and building GPU-accelerated applications. For the `atesting_mi100` partition, users can request up to 3 GPUs and all associated CPU cores (64 max) from a single node for up to one hour. Due to limitations with MIG (see below), we limit users to 1 GPU (with 20 GB of VRAM) and at most 10 CPU cores on the `atesting_a100` partition.  Currently there is no testing partition for the L40 GPUs, however most workflows that successfully test on the `atesting_a100` partition will work on the `al40` partition.

```{important}

The `atesting_a100` partition utilizes NVIDIA's [Multi-Instance GPU (MIG)](https://docs.nvidia.com/datacenter/tesla/mig-user-guide/index.html) feature, which can "slice" GPUs into multiple GPU instances. These GPU instances can be treated as a single GPU. The increase in available GPUs, and in effect increase in GPU access, provided by MIG does come with certain limitations. One important limitation is that MIG does not allow for multiple GPU instances to communicate with each other. This is the reason we limit users to just 1 GPU on the `atesting_a100` partition. For more information on limitations of MIG, please see NVIDIA's MIG [Application Considerations](https://docs.nvidia.com/datacenter/tesla/mig-user-guide/index.html#application-considerations) documentation. 
```

(tabset-ref-gpu-atesting-use)=
`````{tab-set}
:sync-group: tabset-gpu-atesting-use

````{tab-item} Example 1
:sync: gpu-atesting-use-ex1

**Request 1 A100 MIG slice with 10 CPU cores for 30 minutes.**

```bash
sinteractive --partition=atesting_a100 --gres=gpu --ntasks=10 --time=30:00
```

````

```` {tab-item} Example 2
:sync: gpu-atesting-use-ex2

**Request 1 MI100 GPU with 1 CPU core for one hour.**

```bash
sinteractive --partition=atesting_mi100 --gres=gpu:1 --ntasks=1 --time=60:00
```

````

`````

##### `acompile` usage examples:

`acompile` provides near-immediate access to limited resources for the purpose of viewing the module stack, verifying non-MPI jobs, and compiling software. Users can request up to 4 CPU cores (but no GPUs) for a maximum runtime of 12 hours. The partition is accessed with the `acompile` command. Users who need GPU nodes to compile software should use Slurm's `sinteractive` command with the appropriate GPU partition (`ami100` or `aa100`) instead of `acompile`.

(tabset-ref-acompile-use)=
`````{tab-set}
:sync-group: tabset-acompile-use

````{tab-item} Example 1
:sync: acompile-use-ex1

**Get usage information for `acompile`.**

```bash
acompile --help
```

````

```` {tab-item} Example 2
:sync: acompile-use-ex2

**Request 2 CPU cores for 2 hours.**

```bash
acompile --ntasks=2 --time=02:00:00
```

````

`````

# Monitoring Resources

CU Research Computing has two main tools which can help users monitor their HPC resources:
* [Slurmtools](#slurmtools): A [module](./modules.md) that loads a collection of functions to assess recent usage statistics

## Slurmtools
Slurmtools is a collection of helper scripts for everyday use of the [SLURM](https://slurm.schedmd.com/overview.html) job scheduler. Slurmtools can be loaded in as a module from any node (including login nodes). Slurmtools can help us understand the following questions:
* How many core hours (SUs) have I used recently?
* Who is using all of the SUs on my group's account?
* What jobs have I run over the past few days?
* What is my priority?
* How efficient are my jobs?

### __Step 1__: Log in
If you have a CURC account, login as you [normally would](../getting_started/logging-in.md) using your identikey and Duo from a terminal: 

```bash
$ ssh ralphie@login.rc.colorado.edu
```

### __Step 2__: Load the slurm module for the HPC resource you want to query metrics about (Blanca, Alpine):
```bash
$ module load slurm/alpine # substitute alpine for blanca
```

### __Step 3__: Load the `slurmtools` module
```bash
$ module load slurmtools
```

You will see the following informational message:

```
You have sucessfully loaded slurmtools, a collection of functions
 to assess recent usage statistics. Available commands include:

 'suacct' (SU usage for each user of a specified account over N days)

 'suuser' (SU usage for a specified user over N days)

 'seff' (CPU and RAM efficiency for a specified job)

 'seff-array' (CPU, RAM, and time efficiency for a specified array job)

 'jobstats' (job statistics for all jobs run by a specified user over N days)

 'levelfs' (current fair share priority for a specified user)


 Type any command without arguments for usage instructions
 ```

### __Step 4__: Get some metrics!

#### How many Service Units (core hours) have I used?

Type the command name for usage hint:
```bash
$ suuser
```
```
Purpose: This function computes the number of Service Units (SUs)
consumed by a specified user over N days.

Usage: suuser [userid] [days, default 30]
Hint: suuser ralphie 15
```

Check usage for the last 365 days:
```bash
$ suuser ralphie 365
```
```
SU used by user ralphie in the last 365 days:
Cluster|Account|Login|Proper Name|Used|Energy|
alpine|admin|ralphie|Ralphie|15987|0|
alpine|ucb-testing|ralphie|Ralphie|3812|0|
alpine|tutorial1|ralphie|Ralphie|3812|0|
alpine|ucb-general|ralphie|Ralphie|5403|0|
```

This output tells us that:
* Ralphie has used "SUs" across four different accounts over the past year
* Ralphie's usage by account varied from 3,812 SUs to 15,987 SUs


#### Who is using all of the SUs on my groups' account?

Type the command name for usage hint:
```bash
$ suacct
```
```
Purpose: This function computes the number of Service Units (SUs)
consumed by each user of a specified account over N days.

Usage: suacct [account_name] [days, default 30]
Hint: suacct ucb-general 15
```

Check `admin` account usage over past 180 days:
```{tip}
Most user accounts follow the naming convention `ucbXXX_ascX`, in this example we show the `admin` account.
```
```bash
$ suacct admin 180
```
```
SU used by account (allocation) admin in the last 180 days:
Cluster|Account|Login|Proper Name|Used|Energy
alpine|admin|||763240|0
alpine| admin|coke4948|Corey Keasling|84216|0
alpine| admin|frahm|Joel Frahm|24|0
alpine| admin|holtat|Aaron Holt|9832|0
alpine| admin|joan5896|Jonathon Anderson|9357|0
alpine| admin|ralphie|Ralphie|9357|0
```

This output tells us that:
* Five users used the account in the past 180 days.
* Their usage ranged from 24 SUs to 84,216 SUs

#### What jobs have I run over the past few days?

Type the command name for usage hint:
```bash
$ jobstats
```
```
Purpose: This function shows statistics for each job
run by a specified user over N days.

Usage: jobstats [userid] [days, default 5]
Hint: jobstats ralphie 15
```

Check ralphie's jobstats for the past 35 days:
```bash
$ jobstats ralphie 35
```
```
job stats for user ralphie over past 35 days
jobid        jobname  partition    qos          account      cpus state    start-date-time     elapsed    wait
-------------------------------------------------------------------------------------------------------------------
8483382      sys/dash amilan       normal       ucb-gener+   1    TIMEOUT  2021-09-14T09:32:09 01:00:16   0 hrs
8487254      test.sh  amilan       normal       ucb-gener+   1    COMPLETE 2021-09-14T13:21:12 00:00:02   0 hrs
8487256      interact ahub         interacti+   ucb-gener+   1    TIMEOUT  2021-09-14T13:22:11 12:00:22   0 hrs
8508557      acompile acompile     compile      ucb-gener+   2    COMPLETE 2021-09-16T10:41:45 00:00:00   0 hrs
8508561      test.sh  amilan       normal       ucb-gener+   24   CANCELLE 2021-09-22T10:07:03 00:00:00   143 hrs
8508569      test     amilan       normal       ucb-gener+   4096 FAILED   2021-09-16T10:42:46 00:00:00   0 hrs
8508575      test     amilan       normal       ucb-gener+   8192 FAILED   2021-09-16T10:43:17 00:00:00   0 hrs
8508593      test     amilan       normal       ucb-gener+   4096 CANCELLE 2021-09-16T10:44:47 00:00:00   0 hrs
8508604      test     amilan       normal       ucb-gener+   2048 CANCELLE 2021-09-16T10:45:40 00:00:00   0 hrs
8512083      spawner- ahub         interacti+   ucb-gener+   1    TIMEOUT  2021-09-16T16:55:37 04:00:23   0 hrs
8579077      acompile acompile     compile      ucb-gener+   1    COMPLETE 2021-09-24T15:26:32 00:00:47   0 hrs
8627076      acompile acompile     compile      ucb-gener+   24   CANCELLE 2021-10-04T12:17:30 00:10:03   0 hrs
8672525      interact ahub         interacti+   ucb-gener+   1    CANCELLE 2021-10-08T13:29:13 00:07:25   0 hrs
8800741      interact ahub         interacti+   ucb-gener+   1    CANCELLE 2021-10-19T08:11:44 01:48:38   0 hrs
```

This output tells me that:
* Ralphie has run 14 jobs in the past 35 days
* Most jobs had queue waits of < 1 hour
* The number of cores requested ranged from 1-->8192
* The elapsed times ranged from 0 hours to 1 hour and 48 minutes


#### What is my priority?

Type the command name for usage hint:
```bash
$ levelfs
```
```
Purpose: This function shows the current fair share priority of a specified user.
A value of 1 indicates average priority compared to other users in an account.
A value of < 1 indicates lower than average priority
	(longer than average queue waits)
A value of > 1 indicates higher than average priority
	(shorter than average queue waits)

Usage: levelfs [userid]
Hint: levelfs ralphie
```

Check Ralphie's fair share priority:
```bash
$ levelfs ralphie
```
```
ralphie
admin LevelFS: inf
ucb-general LevelFS: 44.796111
tutorial1 LevelFS: inf
ucb-testing LevelFS: inf
```

This output tells me:
* Ralphie hasn't used `admin`, `tutorial1`, or `ucb-testing` for more than a month, and therefore Ralphie has very high ("infinite") priority. 
* Ralphie has used `ucb-general` but not much. Priority is >> 1 , therefore Ralphie can expect lower-than-average queue waits compared to average ucb-general waits.


```{important}
What is "Priority"?
* Your priority is a number between 0.0 --> 1.0 that defines your relative placement in the queue of scheduled jobs
* Your priority is computed each time a job is scheduled and reflects the following factors:
  * Your "Fair Share priority" (the ratio of resources you are allocated versus those you have consumed for a given account)
  * Your job size (slightly larger jobs have higher priority)
  * Your time spent in the queue (jobs gain priority the longer they wait)
  * The partition and qos you choose (this is a minor consideration on CURC systems)
* Your "Fair Share" priority has a half life of 14 days (i.e., it recovers fully in ~1 month with zero usage)
```

#### How efficient are my jobs?

Type the command name for usage hint:
```bash
$ seff
```
```
Usage: seff [Options] <Jobid>
       Options:
       -h    Help menu
       -v    Version
       -d    Debug mode: display raw Slurm data
```

Now check the efficiency of job 8636572:
```bash
$ seff 8636572
```
```
Job ID: 8636572
Cluster: alpine
User/Group: ralphie/ralphiegrp
State: COMPLETED (exit code 0)
Nodes: 1
Cores per node: 24
CPU Utilized: 04:04:05
CPU Efficiency: 92.18% of 04:24:48 core-walltime
Job Wall-clock time: 00:11:02
Memory Utilized: 163.49 MB
Memory Efficiency: 0.14% of 113.62 GB
```

This output tells us that:
* the 24 cores reserved for this job were 92% utilized (anything > 80% is pretty good)
* 163.49 MB RAM was used of 113.62 GB RAM reserved (0.14%). This job is "cpu bound" so the memory inefficiency is not a major issue.

This information is also sent to users who include the `--mail` directive in jobs.

#### How can I check the efficiency of array jobs?

Use the `seff-array` command with the help flag for a usage hint: 
```
$ seff-array -h
```
```
usage: seff-array.py [-h] [-c CLUSTER] [--version] jobid

positional arguments:
  jobid

options:
  -h, --help            show this help message and exit
  -c CLUSTER, --cluster CLUSTER
  --version             show program's version number and exit
```
In order to check the efficiency of all jobs in job array 8636572, run the command: 
```
$ seff-array 8636572
```
This will display the status of all jobs in the array:
```
--------------------------------------------------------
Job Status
COMPLETED: 249
FAILED: 4
PENDING: 1
RUNNING: 22
TIMEOUT: 4
--------------------------------------------------------
```
 Additionally, `seff-array` will display a histogram of the efficiency statistics all of the jobs in the array, separated into 10% increments. For example: 
```
CPU Efficiency (%)
---------------------
+0.00e+00 - +1.00e+01  [  3]  ▌
+1.00e+01 - +2.00e+01  [244]  ████████████████████████████████████████
+2.00e+01 - +3.00e+01  [  8]  █▎
+3.00e+01 - +4.00e+01  [  2]  ▍
+4.00e+01 - +5.00e+01  [  0]
+5.00e+01 - +6.00e+01  [  0]
+6.00e+01 - +7.00e+01  [  0]
+7.00e+01 - +8.00e+01  [  0]
+8.00e+01 - +9.00e+01  [  0]
+9.00e+01 - +1.00e+02  [  0]
```
The above indicates that all of the jobs displayed less than 40% CPU efficiency, with the majority (244/256) demonstrating between 10% and 20% efficiency. This information will also be displayed for memory and time efficiency. 

# Slurm Directive Examples

Below are some examples of SLURM directives that can be used in your batch scripts in order to meet certain job requirements.


(tabset-ref-slurm-dir-exs)=
`````{tab-set}
:sync-group: tabset-slurm-dir-exs

````{tab-item} Example 1
:sync: slurm-dir-exs-1

To run a 32-core job for 24 hours on a single Alpine CPU node:

```bash
#SBATCH --partition=amilan
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=32
#SBATCH --time=24:00:00
```

````

````{tab-item} Example 2
:sync: slurm-dir-exs-2

To run a 56-core job (28 cores/node) across two Alpine CPU nodes in the low-priority qos for seven days:

```bash
#SBATCH --partition=amilan
#SBATCH --nodes=2
#SBATCH --ntasks-per-node=28
#SBATCH --time=7-00:00:00
#SBATCH --qos=long
```

````

````{tab-item} Example 3
:sync: slurm-dir-exs-3

To run a 16-core job for 24 hours on a single Alpine AMD GPU node, using all three GPU accelerators:

```bash
#SBATCH --partition=ami100
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=16
#SBATCH --time=24:00:00
#SBATCH --gres=gpu:3
```

````

````{tab-item} Example 4
:sync: slurm-dir-exs-4

To run a 50-core job for 2 hours on a single Alpine NVIDIA GPU node, using 2 GPUs:

```bash
#SBATCH --partition=aa100
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=42
#SBATCH --time=02:00:00
#SBATCH --gres=gpu:2
```

````
`````     

## Full Example Job Script

Run a 1-hour job on 4 cores on an Alpine CPU node with the normal qos that runs a python script using a custom conda environment.

```
#!/bin/bash

#SBATCH --partition=amilan
#SBATCH --job-name=example-job
#SBATCH --output=example-job.%j.out
#SBATCH --time=01:00:00
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=4
#SBATCH --mail-type=ALL
#SBATCH --mail-user=youridentikey@colorado.edu

module purge
module load anaconda
conda activate custom-env

python myscript.py
```


# Slurm Flags, Partitions, and QoS

Slurm allows the use of flags to specify resources needed for a job. Below is a table describing some of the most common Slurm resource flags, followed by tables describing available partitions and Quality of Service (QoS) options.

## Slurm Resource Flags

Job scripts, the `sbatch` command, and the `sinteractive` command support many different resource requests in the form of flags. These flags are available to all forms of jobs. To review all possible flags for these commands, please visit the [Slurm page on sbatch](http://slurm.schedmd.com/sbatch.html). Below, we have listed some useful flags to consider when running your job script.

| Type                    | Description                                    | Flag                       | Example                       |
| :---------------------- | :--------------------------------------------- | :------------------------- | :---------------------------- |
| [Allocation](../clusters/alpine/allocations.md)  | Specify an allocation account  | `--account=<allocation_name>` <br> | `--account=ucb###_asc1` <br>    |
| Partition          | Specify a partition ([see table below](#partitions)) | `--partition=<partition_name>` <br> | `--partition=amilan` <br>  |
| Sending email      | Receive an email at the beginning or the end of a job | `--mail-type=<type>` <br> | `--mail-type=BEGIN,END` <br>     |
| Email address      | Email address to receive the email                  | `--mail-user=<email_address>`  <br> | `--mail-user=ralphie@colorado.edu` <br>    |
| Number of nodes    | The number of nodes needed to run the job           | `--nodes=<#>` <br>  | `--nodes=1` <br>   |
| Number of tasks    | The ***total*** number of processes needed to run the job | `--ntasks=<#>` <br>  | `--ntasks=4`  <br>  |
| Tasks per node     | The number of processes you wish to assign to each node (only needed for multi-node jobs) | `--ntasks-per-node=<#>` <br> | `--ntasks-per-node=4` <br>  |
| Total memory       | The total memory (per node requested) required for the job. <br> Using `--mem` does not alter the number of cores allocated to the job, but you will be charged for the number of cores corresponding to the proportion of total memory requested. <br> Units of `--mem` can be specified with the suffixes: K,M,G,T (default M)| `--mem=<#><unit (optional)>` <br>  |`--mem=25G` <br>  |
| Quality of service | Specify a QoS ([see table below](#quality-of-service)) | `--qos=<qos_name>` <br>  | `--qos=normal`   <br>   |
| Wall time          | The max amount of time your job will run for        | `--time=<D-HH:MM:SS>`  <br> | `--time=03:00:00` <br>   |
| Job Name           | Name your job so you can identify it in the queue   | `--job-name=<job_name>` <br> | `--job-name=Census-Data-Analysis` <br>   |


## Partitions

Nodes with the same hardware configuration are grouped into partitions. You will need to specify a partition using `--partition` in your job script in order for your job to run on the appropriate type of node. A list of partitions available on Alpine can be found on our [Alpine Hardware](../clusters/alpine/alpine-hardware.md#partitions) page. 

## Quality of Service

Quality of Service (QoS) is used to constrain or modify the characteristics that a job can have. This could come in the form of specifying a QoS to request for a longer run time or a high priority queue for condo owned nodes. For example, by selecting the `long` QoS, a user can place the job in a lower priority queue with a max wall time increased from 24 hours to 7 days. A list of QoS codes available on Alpine can be found on our [Alpine Hardware](../clusters/alpine/alpine-hardware.md#quality-of-service-qos) page. 

# Useful Slurm Commands

Slurm provides a variety of tools that allow a user to manage and
understand their jobs. This tutorial will introduce these tools, as
well as provide details on how to use them.

## Finding queuing information with `squeue`

The `squeue` command is a tool we use to pull up information about the
jobs currently in the Slurm queue. By default, the squeue command will print out the
*__job ID__*, *__partition__*, *__username__*, *__job status__*,
*__number of nodes__*, and *__name of nodes__* for all jobs queued or
running within Slurm. Usually, you wouldn't need information for all
jobs that were queued in the system, so we can specify jobs that only
you are running with the `--user` flag:

```bash
$ squeue --user=your_rc-username
```

We can output non-abbreviated information with the `--long` flag. This
flag will print out the non-abbreviated default information with the
addition of a *__timelimit__* field:

```bash
$ squeue --user=your_rc-username --long
```

The squeue command also provides users with a means to calculate a
job's estimated start time by adding the `--start` flag to our
command. This will append Slurm's estimated start time for each job in
our output information. 

```{note}
The start time provided by this command
can be inaccurate. This is because the time calculated is based on
jobs queued or running in the system. If a job with a higher priority
is queued after the command is run, your job may be delayed.
```


```bash
$ squeue --user=your_rc-username --start
```

When checking the status of a job, you may want to repeatedly call the
squeue command to check for updates. We can accomplish this by adding
the `--iterate` flag to our squeue command. This will run squeue every
n seconds, allowing for a frequent, continuous update of queue
information without needing to repeatedly call squeue:

```bash
$ squeue --user=your_rc-username --start --iterate=n_seconds
```

Press `ctrl`-`c` to stop the command from looping and bring you back
to the terminal.

````{important}
  Do not use an `--iterate=` value less than 60 (i.e. 1 minute). Shorter iterations can overwhelm the Slurm controller and lead to the suspension of RC accounts. 
  ```bash
  $ squeue --user=your_rc-username --start --iterate=60
  ```
````

```{seealso}
For more information on squeue, [visit the Slurm page on
squeue](https://slurm.schedmd.com/squeue.html)
```

## Stopping jobs with `scancel`

Sometimes you may need to stop a job entirely while it’s running. The
best way to accomplish this is with the `scancel` command. The scancel
command allows you to cancel jobs you are running on Research
Computing resources using the job’s ID. The command looks like this:

```bash
$ scancel your_job-id
```

To cancel multiple jobs, you can use a comma-separated list of job IDs:

```bash
$ scancel your_job-id1, your_job-id2, your_jobiid3
```

```{seealso}
For more information, [visit the Slurm manual on scancel](https://slurm.schedmd.com/scancel.html)
```

## Analyzing currently running jobs with `sstat`

The `sstat` command allows users to easily pull up status information
about their currently running jobs. This includes information about *__CPU usage__*,
*__task information__*, *__node information__*, *__resident set size
(RSS)__*, and *__virtual memory (VM)__*. We can invoke the sstat
command as such:

```bash
$ sstat --jobs=your_job-id
```

The default output from `sstat` may not include all the information you need. To remedy this,
we can use the `--format` flag to choose what we want in our
output. The format flag takes a list of comma-separated variables
that specify output data:

```bash
$ sstat --jobs=your_job-id --format=var_1,var_2, ... , var_N
```

A chart of some of these variables is listed in the table below:

Variable    | Description
------------|------------
avecpu      | Average CPU time of all tasks in a job.
averss      | Average resident set size of all tasks.
avevmsize   | Average virtual memory of all tasks in a job.
jobid       | The id of the Job.
maxrss      | Maximum number of bytes read by all tasks in the job.
maxvsize    | Maximum number of bytes written by all tasks in the job.
ntasks      | Number of tasks in a job.

As an example, let's print out a job's id, average cpu time, max
rss, and the number of tasks. We can do this by typing out the command:

```bash
sstat --jobs=your_job-id --format=jobid,cputime,maxrss,ntasks
```

```{seealso}
A full list of variables that specify data handled by sstat can be
found with the `--helpformat` flag or by [visiting the slurm page on
sstat](https://slurm.schedmd.com/sstat.html).
```

## Analyzing past jobs with `sacct`

The `sacct` command allows users to pull up status information about past jobs. This command is very similar to sstat, but is used on jobs that have been previously run on the system instead of currently running jobs. We can pull up accounting information on jobs based on the:

**Job ID:**  
```bash
$ sacct --jobs=your_job-id
```

**Research Computing Username:**
```bash
$ sacct --user=your_rc-username
```

By default, sacct will only pull up jobs that were run on the current day. We can use the `--starttime` flag to tell the command to look beyond its short-term cache of jobs.

```bash
$ sacct –-jobs=your_job-id –-starttime=YYYY-MM-DD
```

To see a non-abbreviated version of sacct output, use the `--long`
flag:

```bash
$ sacct –-jobs=your_job-id –-starttime=YYYY-MM-DD --long
```

### Formatting `sacct` output

Like `sstat`, the standard output of sacct may not provide the
information we want. To remedy this, we can use the `--format` flag to
choose what we want in our output. Similarly, the format flag is
handled by a list of comma-separated variables that specify output
data:

```bash
$ sacct --user=your_rc-username --format=var_1,var_2, ... ,var_N
```

A chart of some of these variables is provided below:

Variable    | Description
------------|------------
account     | Account the job ran under.
avecpu      | Average CPU time of all tasks in the job.
averss      | Average resident set size of all tasks in the job.
cputime     | Formatted (Elapsed time * CPU) count used by a job or step.
elapsed     | Jobs elapsed time formatted as `DD-HH:MM:SS.`
exitcode    | The exit code returned by the job script or salloc.
jobid       | The id of the Job.
jobname     | The name of the Job.
maxdiskread | Maximum number of bytes read by all tasks in the job.
maxdiskwrite| Maximum number of bytes written by all tasks in the job.
maxrss      | Maximum resident set size of all tasks in the job.
ncpus       | Amount of allocated CPUs.
nnodes      | The number of nodes used in a job.
ntasks      | Number of tasks in a job.
priority    | Slurm priority.
qos         | Quality of service.
reqcpu      | Required number of CPUs
reqmem      | Required amount of memory for a job.
user        | Username of the person who ran the job.

As an example, suppose you want to find information about jobs that
were run on March 12, 2024. You want to show information regarding the
job name, the number of nodes used in the job, the number of cpus, the
maxrss, and the elapsed time. Your command would look like this:

```bash
$ sacct --jobs=your_job-id --starttime=2024-03-12 --format=jobname,nnodes,ncpus,maxrss,elapsed
```

As another example, suppose you would like to pull up information on
jobs that were run on February 21, 2024. You would like information on
job ID, job name, QoS, Number of Nodes used, Number of CPUs used,
Maximum RSS, CPU time, Average CPU time, and elapsed time. Your
command would look like this:

```bash
$ sacct –-jobs=your_job-id –-starttime=2024-02-21 --format=jobid,jobname,qos,nnodes,ncpu,maxrss,cputime,avecpu,elapsed
```

```{seealso}
A full list of variables that specify data handled by sacct can be
found with the `--helpformat` flag or by [visiting the slurm page on
sacct](https://slurm.schedmd.com/sacct.html).
```

## Controlling queued and running jobs using `scontrol`

The `scontrol` command provides users extended control of their jobs
run through Slurm. This includes actions like suspending a job,
holding a job from running, or pulling extensive status information on
jobs.

To suspend a job that is currently running on the system, we can use
scontrol with the `suspend` command. This will stop a running job on
its current step that can be resumed at a later time. We can suspend a
job by typing the command:

```
$ scontrol suspend job_id
```

To resume a paused job, we use scontrol with the `resume` command:

```bash
$ scontrol resume job_id
```

Slurm also provides a utility to hold jobs that are queued in the
system. Holding a job will place the job in the lowest priority,
effectively "holding" the job from being run. A job can only be held
if it's waiting on the system to be run. We use the `hold` command to
place a job into a held state:

```bash
$ scontrol hold job_id
```

We can then release a held job using the `release` command:

```bash
$ scontrol release job_id
```

`scontrol` can also provide information on jobs using the `show job`
command. The information provided by this command is quite extensive
and detailed, so be sure to either clear your terminal window, grep
certain information from the command, or pipe the output to a separate
text file:

```bash
# Output to console
$ scontrol show job job_id

# Streaming output to a textfile
$ scontrol show job job_id > outputfile.txt

# Piping output to Grep and find lines containing the word "Time"
$ scontrol show job job_id | grep Time
```

```{seealso}
 - For a full primer on grep and regular expressions, [visit GNU's page
on Grep](https://www.gnu.org/software/grep/manual/grep.html).

 - For more information on scontrol, [visit the Slurm page on
scontrol](https://slurm.schedmd.com/scontrol.html).
```

# `squeue` Status and Reason Codes

The `squeue` command details a variety of information on an active
job’s status with state and reason codes. *__Job state
codes__* describe a job’s current state in queue (e.g. pending,
completed). *__Job reason codes__* describe the reason why the job is
in its current state. 

The following tables outline a variety of job state and reason codes you
may encounter when using squeue to check on your jobs.

## Job State Codes

| Status        | Code  | Explaination                                                           |
| ------------- | :---: | ---------------------------------------------------------------------- |
| COMPLETED	| `CD`	| The job has completed successfully.                                    |
| COMPLETING	| `CG`	| The job is finishing but some processes are still active.              |
| FAILED	| `F`	| The job terminated with a non-zero exit code and failed to execute.    |
| PENDING	| `PD`	| The job is waiting for resource allocation. It will eventually run.    |
| PREEMPTED	| `PR`	| The job was terminated because of preemption by another job.           |
| RUNNING	| `R`	| The job currently is allocated to a node and is running.               |
| SUSPENDED	| `S`	| A running job has been stopped with its cores released to other jobs.  |
| STOPPED	| `ST`	| A running job has been stopped with its cores retained.                |

```{seealso}
A full list of these Job State codes can be found in [Slurm’s
documentation.](https://slurm.schedmd.com/squeue.html#lbAG)
```

## Job Reason Codes

| Reason Code              | Explanation                                                                                |
| ------------------------ | ------------------------------------------------------------------------------------------- |
| `Priority`	           | One or more higher priority jobs is in queue for running. Your job will eventually run.     |
| `Dependency`	           | This job is waiting for a dependent job to complete and will run afterward.                |
| `Resources`	           | The job is waiting for resources to become available and will eventually run.               |
| `InvalidAccount`	   | The job’s account is invalid. Cancel the job and rerun with the correct account.             |
| `InvaldQoS`              | The job’s QoS is invalid. Cancel the job and rerun with the correct account.                 |
| `QOSGrpCpuLimit` 	   | All CPUs assigned to your job’s specified QoS are in use; the job will run eventually.          |
| `QOSGrpMaxJobsLimit`	   | Maximum number of jobs for your job’s QoS have been met; the job will run eventually.           |
| `QOSGrpNodeLimit`	   | All nodes assigned to your job’s specified QoS are in use; the job will run eventually.         |
| `PartitionCpuLimit`	   | All CPUs assigned to your job’s specified partition are in use; the job will run eventually.    |
| `PartitionMaxJobsLimit`  | Maximum number of jobs for your job’s partition have been met; the job will run eventually.     |
| `PartitionNodeLimit`	   | All nodes assigned to your job’s specified partition are in use; the job will run eventually.   |
| `AssociationCpuLimit`	   | All CPUs assigned to your job’s specified association are in use; the job will run eventually.  |
| `AssociationMaxJobsLimit`| Maximum number of jobs for your job’s association have been met; the job will run eventually.   |
| `AssociationNodeLimit`   | All nodes assigned to your job’s specified association are in use; the job will run eventually. |

```{seealso}
A full list of these Job Reason Codes can be found [in Slurm’s
documentation.](https://slurm.schedmd.com/squeue.html#lbAF)
```

# Alpine Allocations

## What are allocations and why do I need one?

In the simplest terms, an allocation is a way for us to specify your cut 
of Alpine's computational resources. Allocations are referred to as 
accounts in Slurm's documentation and are indicated by the `--account` 
directive:

```
#SBATCH --account=______
```

Allocations are required to run on CURC clusters. They help us keep track 
of system usage for reporting purposes and to ensure we have enough 
resources to accommodate all of our users.  

## FairShare, Priority, and Allocations

### Fairshare Scheduling
The idea behind fairshare scheduling is simple, even though its
implementation is complex: jobs submitted by people who have underutilized
their allocated resources get higher priority, while jobs submitted by
people who have overutilized their allocated resources get lower priority.

### Level Fairshare
A Level Fairshare (`LevelFS`) is a value calculated by [Slurm's Fairshare 
Algorithm](https://slurm.schedmd.com/fair_tree.html). A user's 
assigned shares (determined by their allocation) and usage (based on their 
job history) contribute to their `LevelFS` value. Information on how to 
check your `LevelFS` score can be found on the 
["How can I see my current FairShare priority?"](../../getting_started/faq.md#how-can-i-see-my-current-fairshare-priority) section of our FAQ page.

```{note}
If there are no other pending jobs, and enough resources are 
available, then your job will run regardless of your previous usage.
```

### Priority Score
When you request resources on Alpine, your job's priority determines its 
position in the queue relative to other jobs. A job’s priority is based on 
multiple factors, including (but not limited to) FairShare score, job age, 
resources requested, job size, and QOS. 

# Batch Jobs and Job Scripting

Batch jobs are, by far, the most common type of job on our HPC system. Batch jobs are resource provisions that run applications on compute nodes and do not require supervision or interaction. Batch jobs are commonly used for applications that run for long periods of time or require little to no user input. 

## Job Scripts

Even though it is possible to run jobs completely from the command line, it is often overly tedious and unorganized to do so. Instead, Research Computing recommends constructing a job script for your batch jobs. A **job script** is a set of Linux commands paired with a set of resource requirements that can be passed to the Slurm job scheduler. Slurm will then generate a job according to the parameters set in the job script. Any commands that are included with the job script will be run within the job.

## Running a Job Script

Running a job script can be done with the `sbatch` command:

```bash
sbatch <your-job-script-name>
```

Because job scripts specify the desired resources for your job, you won't need to specify any resources on the command line. You can, however, overwrite or add any job parameter by providing the specific resource as a flag within `sbatch` command:

```bash
sbatch --partition=amilan <your-job-script>
```

Running this command would force your job to run on the amilan partition *no matter what your job script specified*.

## Making a Job Script

Although Research Computing provides a variety of different sample scripts users can utilize when running their own jobs, knowing how to draft a job script can be quite handy if you need to debug any errors in your jobs or you need to make substantial changes to a script.

A job script looks something like this:

```bash
#!/bin/bash

#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --time=00:10:00
#SBATCH --partition=atesting
#SBATCH --qos=testing
#SBATCH --output=sample-%j.out

module purge

module load intel
module load mkl

echo "== This is the scripting step! =="
sleep 30
./executable.exe
echo "== End of Job =="
```

Normally job scripts are divided into 3 primary parts: directives, loading software, and user scripting. Directives give the terminal and the Slurm daemon instructions on setting up the job. Loading software involves cleaning out the environment and loading specific pieces of software you need for your job. User scripting is simply the commands you wish to be executed in your job.  

### 1. Directives

A directive is a comment that is included at the top of a job script that tells the shell information about the script. 

The first directive, the shebang directive, is always on the first line of any script. The directive indicates which shell you want running commands in your job. Most users employ bash as their shell, so we will specify bash by typing:

```bash
#!/bin/bash
```

The next directives that must be included with your job script are *sbatch* directives. These directives specify resource requirements to Slurm for a batch job.  These directives must come after the shebang directive and before any commands are issued in the job script. Each directive contains a flag that requests a resource the job would need to complete execution. An sbatch directive is written as such:

```bash
#SBATCH --<resource>=<amount>
```

For example, if you wanted to request 2 nodes with an sbatch directive, you would write:

```bash
#SBATCH --nodes=2
```

```{seealso}
A list of some useful sbatch directives can be found on the [Slurm Flags, Partitions, and QoS](job-resources.md) page. A full list of commands can be found in Slurm's [documentation for sbatch](https://slurm.schedmd.com/sbatch.html).
```

### 2. Software

Because jobs run on different nodes, any shared software that is needed must be loaded via the job script. Software can be loaded in a job script just like it would be on the command line. First, we will purge all software that may be left behind from your working environment on a compile node by running the command:

```bash
module purge
```

Next, you can load whatever software you need by running the following command:

```bash
module load <software>
```

```{seealso}
More information about software modules can be found in the [Modules System](../compute/modules.md) page.
```

### 3. User Scripting

The last part of a job script is the actual script. This includes all user commands that are needed to set up and execute the desired task. Any Linux command can be utilized in this step. Scripting can range from highly complex loops iterating over thousands of files to a simple call to an executable. Below is a simple example of some user scripting:

```bash
echo "== This is the scripting step! =="

touch tempFile1.in
touch tempFile2.in

sleep 30
./executable.exe tempfile1.in tempfile2.in

echo "== End of Job =="
```

## Examples

(tabset-ref-batch-scripting)=
`````{tab-set}
:sync-group: tabset-batch-scripting

````{tab-item} Example 1 
:sync: batch-scripting-ex1

5 minutes, 1 node, 1 core C++ Job:

```bash
#!/bin/bash

#SBATCH --nodes=1
#SBATCH --time=00:05:00
#SBATCH --partition=atesting
#SBATCH --qos=testing
#SBATCH --ntasks=1
#SBATCH --job-name=cpp-job
#SBATCH --output=cpp-job.%j.out

module purge
module load gcc

./example_cpp.exe
```

````
````{tab-item} Example 2
:sync: batch-scripting-ex2

7 minutes, 1 node, 4 cores C++ OpenMP Job:

```bash
#!/bin/bash

#SBATCH --nodes=1
#SBATCH --time=00:07:00
#SBATCH --partition=atesting
#SBATCH --qos=testing
#SBATCH --ntasks=4
#SBATCH --job-name=omp-cpp-job
#SBATCH --output=omp-cpp-job.%j.out

module purge
module load gcc

export OMP_NUM_THREADS=4

./example_omp.exe
```

````

````{tab-item} Example 3
:sync: batch-scripting-ex3

10 minutes, 2 nodes, 16 cores C++ MPI Job:

```bash
#!/bin/bash

#SBATCH --nodes=2
#SBATCH --time=00:10:00
#SBATCH --partition=atesting
#SBATCH --qos=testing
#SBATCH --ntasks=16
#SBATCH --job-name=mpi-cpp-job
#SBATCH --output=mpi-cpp-job.%j.out

module purge
module load intel
module load impi

mpirun -np 16 ./example_mpi.exe
```
````
`````

# Example Application Job Building: VASP

The Vienna Ab initio Simulation Package ([VASP](https://www.vasp.at)) is a computer program for atomic scale materials modelling, e.g. electronic structure calculations and quantum-mechanical molecular dynamics

VASP requires a license. Individual students or entire departments/faculties are not eligible, and therefore CU Research Computing does not have a VASP module for community use. Licenses are issued to well-defined research groups under the direction of a single chair, professor, or group leader at one single physical location.  Group leaders may [apply for a VASP license](https://www.vasp.at/sign_in/registration_form/), after which they will be given access to the source code. 

The documentation below demonstrates how to install and use VASP in one's `/projects/$USER` directory.  A typical case would be to install the software in the `/projects/$USER` directory of the group leader, and then make it available to group members by emailing rc-help@colorado.edu to request that they be added to the Linux user group of the group leader. 

## Prerequisites
* you have a copy of the source code
* you are in a group that has a vasp license
* you only use VASP for research purposes

## Assumptions
* the example below is for version 5.4.4; adjust version to match yours
* the example below assumes the source code is in a tar.gz file; if the source code is in a directory, you can skip the "tar -xf" step
* the example below assumes you will install the software in /projects/$USER/software; adjust as needed.
* you have started an interactive job on alpine ("module load slurm/alpine; acompile")

## To compile vasp
```bash
module purge
module load intel/2022.1.2
module load impi/2021.5.0
module load mkl/2022.0.2
cd /projects/$USER/software
tar -xf vasp.5.4.4.tar.gz
cd vasp.5.4.4
cp arch/makefile.include.linux_intel ./makefile.include
make
```

## To use vasp (example job script)

```bash
#!/bin/bash

#SBATCH --partition=amilan
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=2
#SBATCH --time=1:00:00
#SBATCH --output=vasp.%j.out
#SBATCH --job-name=vasp
#SBATCH --constraint=ib

# this example draws on a the vasp tutorial at:
#    https://www.vasp.at/tutorials/latest/bulk/part1/

#download and unzip the tutorial files
#and set up the POSCAR file (change "a" to "3.9")
wget https://www.vasp.at/tutorials/latest/bulk-part1.zip
tar -xf bulk-part1.zip
unzip bulk-part1.zip
cd bulk-part1/e01_fcc-Si/
sed -i 's/a/3.9/g' ./POSCAR

#load the required modules
module purge
module load intel/2022.1.2
module load impi/2021.5.0
module load mkl/2022.0.2

# add the vasp bin directory to your path
export PATH=$PATH:/projects/$USER/software/vasp.5.4.4/bin

# run vasp
mpirun -n ${SLURM_NTASKS} vasp_std
```

# Example Application Job Building: AlphaFold

## AlphaFold Overview
AlphaFold is a program that predicts the three-dimensional structure of proteins from their amino acid sequences. AlphaFold 2 and AlphaFold 3 are available as modules on both Alpine and Blanca. For detailed instructions on running each version, please select the relevant tab below.

(tabset-ref-batch-scripting)=
`````{tab-set}
:sync-group: tabset-batch-scripting

````{tab-item} AlphaFold 2 
:sync: batch-scripting-ex1

Load the default AlphaFold 2 module:
```
module load alphafold/2.3.1
```

View run options:
```
run_alphafold
```
#### AlphaFold 2 Module
Loading the AlphaFold 2 module does the following:

- redirects temporary files from `/tmp` to `/scratch/alpine/$USER`
    - you can override this path by resetting TMPDIR *after* you load the module:
        ```
        module load alphafold/2.3.1
        export TMPDIR=<path/of/your/choosing>
        ```
- activates the AlphaFold 2 conda environment

- sets `CURC_AF_DBS` and `CURC_AF_EXAMPLES` environment variables (see "AlphaFold 2 Databases" and "AlphaFold 2 Examples" sections, below)

- creates a shortcut to the AlphaFold 2 script so you can run the program with `run_alphafold`

#### AlphaFold 2 Databases
The AlphaFold 2 databases are located in `/gpfs/alpine1/datasets/bioinformatics/alphafold`.
Note that this directory is not visible from a login node. Loading the AlphaFold 2 module stores this path in `CURC_AF_DBS`.

#### AlphaFold 2 Examples
Several example fasta files are located in `/curc/sw/install/bio/alphafold/examples`.
Loading the AlphaFold 2 module stores this path in `CURC_AF_EXAMPLES`:

```
ls $CURC_AF_EXAMPLES
dummy.fasta  multimer.fa  rcsb_pdb_7DDD.fasta  T1050.fasta
```
#### Example Job Script
This example job script below is saved in `/curc/sw/install/bio/alphafold/2.3.1`. You can copy it to any space you have write permissions and make the desired changes:
```bash
cd /projects/$USER
cp /curc/sw/install/bio/alphafold/2.3.1/alphafold_alpine.sh .
```

``` bash
#!/bin/bash

#SBATCH --nodes=1
#SBATCH --time=06:00:00
#SBATCH --partition=aa100
#SBATCH --qos=normal
#SBATCH --gres=gpu:1
#SBATCH --job-name=multimer_test
#SBATCH --output=multimer_test_%j.out
#SBATCH --ntasks=40
#SBATCH --mail-type=ALL
#SBATCH --mail-user=<your email address>

module purge
module load alphafold/2.3.1

#change directory
cd /projects/$USER

#run AlphaFold
run_alphafold -d $CURC_AF_DBS -o . -f $CURC_AF_EXAMPLES/dummy.fasta -t 2020-05-14 -m "monomer" -g true
```

````
````{tab-item} AlphaFold 3
:sync: batch-scripting-ex2

AlphaFold 3 has a substantially updated diffusion-based architecture that is capable of predicting the joint structure of complexes including proteins, nucleic acids, small molecules, ions and modified residues. That neccessitates a different kind of input than the fasta input in AlphaFold 2.

On CURC’s Alpine system, AlphaFold 3 is available as a containerized module. It uses Apptainer/Singularity under the hood and is fully self-contained except for the separately downloaded model parameters (required).

#### AlphaFold 3 Module
Load AlphaFold 3 module:
```
module load alphafold/3.0.0
```
View run options:
```
run_alphafold --help
```
Loading the AlphaFold 3 module does the following:
- sets environment variables used by the wrapper script:
    - `AF3_IMAGE`: Path to the AlphaFold 3 container image 
    - `AF3_CODE_DIR`: Directory containing the AlphaFold 3 codebase
    - `AF3_DATABASES_DIR`: Location of the required AlphaFold 3 reference databases

- redirects temporary files to `/scratch/alpine/$USER`
    - you can override this path by resetting TMPDIR *after* you load the module:
        ```
        module load alphafold/3.0.0
        export TMPDIR=<path/of/your/choosing>
        ```
- creates a shortcut to the AlphaFold 3 script so you can run the program with `run_alphafold`

#### AlphaFold 3 Model Weights
```{important}
Due to license restrictions for AlphaFold 3 model weights, you must read and comply with the [Model Parameters](https://github.com/google-deepmind/alphafold3/blob/main/WEIGHTS_TERMS_OF_USE.md) and [Outputs](https://github.com/google-deepmind/alphafold3/blob/main/OUTPUT_TERMS_OF_USE.md) Terms of Use. In short, only non-profit activity is allowed, unethical use of the outputs is disallowed and make sure to cite the AlphaFold 3 paper in any publication. To gain access to AlphaFold 3 at CURC, request access to the weights by filling out [this form](https://docs.google.com/forms/d/e/1FAIpQLSfWZAgo1aYk0O4MuAXZj8xRQ8DafeFJnldNOnh_13qAx2ceZw/viewform). You will receive two e-mails. First is acknowledgement of receipt of the request form. The second, in a day or so, is the approval with a link to download the weights. Once you have downloaded them, put them in a filesystem you have access to on Alpine.
You will need to specify the path to the directory where you save the model weights using the `--model_dir=<path to weights>`.

```

#### AlphaFold 3 Input
AlphaFold 3 uses JSON input files instead of FASTA.
You can either:
- Provide a single JSON file via `--json_path=<path of input>`
- Or a directory of JSONs via `--input_dir=<path of input>`

#### AlphaFold 3 Databases
Databases used by AlphaFold 3 are pre-installed and accessible via:
`/gpfs/alpine1/datasets/bioinformatics/alphafold3`. Note that this directory is not visible from a login node. Loading the AlphaFold 3 module stores this path in `AF3_DATABASES_DIR`.

#### AlphaFold 3 Workflow
AlphaFold 3 runs in two stages:

Stage 1 (MSA Search): CPU and I/O-intensive; uses jackhmmer and hhmsearch.

Stage 2 (Inference): GPU-intensive; performs structure prediction.

To better utilize limited GPU resources, these stages can be split using flags:
 - `--norun_inference` → Run only the MSA/data pipeline (Stage 1)
 - `--norun_data_pipeline` → Run only the inference step (Stage 2)

#### AlphaFold 3 Examples
Example input files and scripts are in `/curc/sw/install/bio/alphafold/3.0.0/examples`.
Loading the AlphaFold 3 module stores this path in `AF3_EXAMPLES`:
```
ls $AF3_EXAMPLES
alphafold3_alpine_cpu.sh  alphafold3_alpine_gpu.sh  alphafold3_alpine.sh  fold_protein_2PV7
```
This folder includes:
- `alphafold3_alpine.sh`: Sample batch script to run the complete AlphaFold 3 pipeline.
- `alphafold3_alpine_cpu.sh`: Sample batch script to run only Stage 1(MSA Search).
- `alphafold3_alpine_gpu.sh`: Sample batch script to run only Stage 2 (Inference).

You can copy the examples folder to a location where you have write permissions and customize the scripts:

```bash
cd /projects/$USER
cp -R /curc/sw/install/bio/alphafold/3.0.0/examples .
cd examples
```

#### Example Job Script
Path of the script: `$AF3_EXAMPLES/alphafold3_alpine.sh`

``` bash
#!/bin/bash

#SBATCH --nodes=1
#SBATCH --time=30:00
#SBATCH --partition=al40
#SBATCH --qos=normal
#SBATCH --gres=gpu:1
#SBATCH --job-name=af3_test
#SBATCH --output=af3_test_%j.out
#SBATCH --ntasks=8
#SBATCH --mail-type=ALL
#SBATCH --mail-user=<your email address>

# Load the AlphaFold 3 module
module purge
module load alphafold/3.0.0

# Set input JSON, output directory, and model parameter path
export INPUT_FILE=$AF3_EXAMPLES/fold_protein_2PV7/alphafold_input.json
export OUTPUT_DIR=/path/to/output
export AF3_MODEL_PARAMETERS_DIR=/path/to/alphafold3/params

# Run AlphaFold 3
run_alphafold --json_path=$INPUT_FILE --output_dir=$OUTPUT_DIR --model_dir=$AF3_MODEL_PARAMETERS_DIR
```
````
`````

# MPI Best practices
MPI, or Message Passing Interface, is a powerful library standard that allows for the parallel execution of applications across multiple processors on a system. It differs from other parallel execution libraries like OpenMP by also allowing a user to run their applications across multiple nodes. Unfortunately it can sometimes be a bit tricky to run a compiled MPI application within an HPC resource. The following page outlines best practices in running your MPI applications across CURC resources.  

```{attention}
Please note that this page *does not* cover compiling or optimization of MPI applications.  
```

## MPI Compatible Compilers and Libraries

### Selecting your Compiler and MPI

Several families of compilers are available to users: Intel, GCC, and AOCC _(Alpine only)_.  Intel compilers have Intel MPI available for messsage passing, and GCC and AOCC compilers have OpenMPI available for message passing. To load a compiler/MPI combo run one the following commands from a job script or compile node (note that you should subsitute the version you need for `<version>` in the examples below; available compiler versions can be seen by typing `module avail`):


(tabset-ref-mpi-best-compiler)=
`````{tab-set}
:sync-group: tabset-mpi-best-compiler

````{tab-item} Intel
:sync: mpi-best-compiler-intel

```bash
module load intel/<version> impi
```

````

````{tab-item} GCC
:sync: mpi-best-compiler-gcc

```bash
module load gcc/<version> openmpi

# Uncomment this additional line when adding this command to a JobScript!
# SLURM_EXPORT_ENV=ALL
```

````

````{tab-item} AOCC
:sync: mpi-best-compiler-aocc

```bash
module load aocc/<version> openmpi

# Uncomment this additional line when adding this command to a JobScript!
# SLURM_EXPORT_ENV=ALL
```

````

`````

```{important}
It is important to note that use of OpenMPI should be paired with the `SLURM_EXPORT_ENV=ALL` environment variable to ensure the job can function when scheduled from a login node!
```

```{note}
On Blanca, in most situations you will want to try to compile and run your applications utilizing the Intel set of compilers and MPI libraries. Most CPUs on Blanca are of Intel architecture, so utilizing Intel will ensure the highest level of optimization comes from your compiler. GCC should only be utilized when your application cannot be compiled on intel software or if compiler specific optimizations exist within your code. We do not yet have compiler/MPI recommendations for Alpine, which has AMD CPUs. 
```

## Commands to Run MPI Applications
Regardless of compiler or MPI distribution, there are 3 “wrapper” commands that will run MPI applications: `mpirun`, `mpiexec`, and `srun`. These “wrapper” commands should be used after loading in your desired compiler and MPI distribution and simply prepend whatever application you wish to run. Each command offers their own pros and cons alongside nuance as to how they function.


(tabset-ref-mpi-best-prac-run)=
`````{tab-set}
:sync-group: tabset-mpi-best-prac-run

````{tab-item} mpirun
:sync: mpi-best-prac-run-mpirun

`mpirun` is probably the most direct method to run MPI applications with the command being tied to the distribution. This means distribution dependent flags can be passed directly through the command.   

```bash
mpirun -np <core-count> ./<your-application>
```

````

````{tab-item} mpiexec
:sync: mpi-best-prac-run-mpiexec

`mpiexec` is a standardized MPI command execution command that allows for more general MPI flags to be passed. This means that commands are universal across all distributions.

```bash
mpiexec -np <core-count> ./<your-application>
```

````

````{tab-item} srun
:sync: mpi-best-prac-run-srun

The final command `srun` is probably the most abstracted away from a specific implementation. This command lets Slurm figure out specific MPI features that are available in your environment and handles running the process as a job. This command is usually a little less efficient and may have some issues with reliability. 

```bash
srun -n <core-count> ./<your-application>
```

````
`````

```{note}
RC usually recommends `mpirun` and `mpiexec` for simplicity and reliability when running MPI applications. `srun` should be used sparingly to avoid issues with execution.
```

## Running MPI on Alpine

Running MPI jobs on Alpine is relatively straightforward. However, one caveat on Alpine is that MPI jobs cannot be run across chassis, which limits them to a maximum `--ntask` count of 4096 cores (64 nodes per chassis * 64 cores each).

Simply select the Compiler and MPI wrapper you wish to use and place it in a job script. In the following example, we run a 128 core, 4 hour job with a gcc compiler and OpenMPI:  

```
#!/bin/bash
#SBATCH --nodes=2
#SBATCH --time=04:00:00
#SBATCH --partition=amilan
#SBATCH --qos=normal
#SBATCH --constraint=ib
#SBATCH --ntasks=128
#SBATCH --job-name=mpi-job
#SBATCH --output=mpi-job.%j.out

module purge
module load gcc/10.3 openmpi
  
export SLURM_EXPORT_ENV=ALL

#Run a 128 core job across 2 nodes:
mpirun -np $SLURM_NTASKS /path/to/mycode.exe

#Note: $SLURM_NTASKS has a value of the amount of cores you requested
```

```{important}
When running MPI jobs on Alpine, you can use the `--constraint=ib` flag to force the job onto an Alpine node that has Infiniband, the networking fabric used by MPI.

To ensure optimal MPI performance and proper task placement, always explicitly specify the number of nodes with the `--nodes` flag. For example:
- Use `--nodes=1` if you're using up to 64 cores (one full node).
- Use `--nodes=2` and `--ntasks=128` for 128-core jobs.
Continue scaling by full nodes to maintain efficient communication (e.g., nodes=4 for 256 tasks, etc.).
```

## Running MPI on Blanca

Blanca is often a bit more complicated due to the variety of nodes available. In general, there are 3 types of nodes on Blanca that can all run single node multi-core MPI processes that may require additional flags and parameters to achieve cross node parallelism.  

```{important}
As with Alpine, it's recommended to explicitly specify the number of nodes using `nodes` along with `ntasks`, especially for multi-node MPI jobs. Blanca nodes may have different core counts depending on the hardware configuration, so be sure to match your `ntasks` to the number of cores available per node. You can check a node's core count using `scontrol show node <node-name>`.
```

### General Blanca Nodes
General Blanca nodes are not intended to run multi-node processes but this can still be achieved through the manipulation of some network fabric settings. In order to achieve cross node parallelism we must force MPI to utilize ethernet instead of our normal high speed network fabric. We can enforce this with various `mpirun` flags for each respective compiler.


(tabset-ref-mpi-best-prac-blanca)=
`````{tab-set}
:sync-group: tabset-mpi-best-prac-blanca

````{tab-item} Intel Single-Node Jobs
:sync: mpi-best-prac-blanca-intel-sing

```bash
mpirun -genv I_MPI_FABRICS=shm
```
````

````{tab-item} Intel Multi-Node Jobs
:sync: mpi-best-prac-blanca-intel-mult

Constrain Jobs to EDR IB (InfiniBand)

```bash
mpirun -genv I_MPI_FABRICS=edr
```

````

````{tab-item} Open MPI
:sync: mpi-best-prac-blanca-openmpi

```bash
mpirun --mca btl tcp <other arguments>
```

````


`````

```{note}
This does not ensure high speed communications in message passing, but it will allow for basic parallelization across nodes.
```

### Blanca HPC
Blanca HPC comes equipped with InfiniBand high speed interconnects that allow for high speed communication between nodes. These nodes supoort the Intel and Intel MPI compiler/MPI combo, as well as the `gcc`/`openmpi_ucx` modules _(note: bve sure to use the *ucx* version of the OpenMPI module)_. 

Blanca HPC nodes can easily be distinguished from other Blanca nodes with the node's name in the cluster. Nodes will clearly be distinguished with the `bhpc` prefix.  They also will have the `edr` feature in their feature list if you query them with `scontrol show node`. If you are using Open MPI, jobs on  Blanca HPC nodes can be run using `mpirun` without any special arguments, although be sure to `export SLURM_EXPORT_ENV=ALL` prior to invoking `mpirun`.  If you are using IMPI, select the `ofa` (Open Fabrics Alliance) option to enable Infiniband-based message passing, the fastest interconnect availble on the `bhpc` nodes. You can do this with the following flag: 

```
mpirun -genv I_MPI_FABRICS shm:ofa <other arguments>
```
  
 
### ROCE Enabled Nodes
The nodes in Blanca chassis 5 (nodes named `bnode05<NN>`) are equipped with high speed network fabrics that are more suited for cross node MPI processes. These nodes are labeled as *RoCE enabled* and require applications to be compiled with UCX-enabled openmpi modules, which are available with both `gcc/8.2.0` and `gcc/10.2.0`.

If you are unsure if your node supports RoCE feature then you can check by using the scontrol command on your node.  

```
scontrol show node <your-bnode>
```

You will be presented a block information that details all the nodes features. The key feature you should look for is `fdr`. If your Blanca node lacks this feature then it is not ROCE Enabled.  Jobs on RoCE nodes can be run using `mpirun` without any special arguments, although be sure to `export SLURM_EXPORT_ENV=ALL` prior to invoking `mpirun`. 

# Load Balancer

The CU Research Computing Load Balancer is an effective tool for
optimally utilizing multiple processors and nodes on the CURC HPC
resources, without the need to learn OpenMP or MPI. This document
assumes user knowledge of Slurm jobs, shell scripting, and
some python.


## Why Use the Load Balancer?

Suppose you have a very simple serial program that crops a photo, and
you need to apply it to crop several million photos. You could rewrite
the serial program into a parallel program that would utilize multiple
processors to more quickly run the program over the entire set of
photos (compared to doing one-at-a-time), but this would require some
knowledge of parallel programming. Even worse, if your code is in a
language that has limited parallelization capabilities, so this may not
be an option. The easiest solution for this problem is to utilize the
Load Balancer.


## Using the Load Balancer

The Load Balancer is a tool provided by CU Boulder Research Computing
that allows shell commands (for example, calls to serial programs) to
be distributed amongst nodes and cores on CURC clusters. This means code
doesn’t need to be explicitly parallelized for MPI or
OpenMP. Additionally, code can be written in any language that can be
run from a Linux shell.

Let’s create a simple ‘Hello World’ serial python script to
demonstrate the Load Balancer tool. We will call the script
`hello_World.py` and it will print “Hello World from process: ”
followed by a command line argument:

```python
import sys

print ("Hello World from process: ", sys.argv[1])
```

Now we will create a list of calls to the python script that will be
distributed to multiple cores. (Each compute node has one or more
discrete compute processor; most modern processors are made up of
multiple compute "cores", each of which can operate independently and
simultaneously.)

Instead of slowly typing out commands one-at-a-time, we will use a
bash shell script to create our commands. In a text editor, create a
bash shell script called `create_hello.sh`, that has the following
text:

```bash
#!/bin/bash

for i in {1..4}
do
  echo "python hello_World.py $i;" >> lb_cmd_file
done
```

Next run the bash script by first changing permissions of the script
to be executable by typing: `chmod +x create_hello.sh` and then by
typing: `./create_hello.sh` at the terminal prompt. It will create a
file called `lb_cmd_file` that contains 4 calls to our
`hello_World.py` script:

```bash
python3 hello_World.py 1;
python3 hello_World.py 2;
python3 hello_World.py 3;
python3 hello_World.py 4;
```

Now create a job script called `run_hello.sh` that will run all instances of your python script in `lb_cmd_file` with the Load Balancer. Within the script, in addition to specifying the `loadbalance` module, we may need to load other software modules or an [anaconda environment we previously built](./python.md), in order to access whatever software we will be running with the Load Balancer. Your job script should look something like this:

```bash
#!/bin/bash

#SBATCH --nodes=1
#SBATCH --time 00:02:00
#SBATCH --partition atesting
#SBATCH --qos testing
#SBATCH --ntasks=4
#SBATCH --job-name lbPythonDemo
#SBATCH --output loadbalance.out

module purge

# Load the Load Balancer module *first*
module load loadbalance/0.2

# Now load any other software modules you need, e.g.:
# module load anaconda 
# conda activate my_python_env

# now run your workflow! 
$CURC_LB_BIN/mpirun lb lb_cmd_file
```

Running this script via `sbatch run_hello.sh` will run the commands we stored in
lb_cmd_file in parallel. A successful job will result in output that
looks something like this:

```
Hello World from process: 2
Hello World from process: 1
Hello World from process: 4
Hello World from process: 3
```

```{note}
* The user must ensure they load the `loadbalance` module _before_ loading any other modules.
* The user should invoke loadbalance with `$CURC_LB_BIN/mpirun lb your-command-file` as shown in the example above.
* The `loadbalance` module uses 1 core as a workflow manager.  Therefore, if you request, e.g., 8 cores, the Load Balancer will employ 1 core to manage your workflow tasks across 7 cores.
``` 

## Additional Resources

* [https://www.inspirenignite.com/load-balancing-in-parallel-computers/](https://www.inspirenignite.com/load-balancing-in-parallel-computers/)
